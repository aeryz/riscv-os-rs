//! ## Linked list allocator

use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};

use crate::KernelAllocator;

#[repr(C)]
pub struct LinkedListAllocator {
    start_addr: usize,
    end_addr: usize,
    head: *mut Header,
}

#[repr(C)]
struct Header {
    sz: usize,
    free: bool,
    next: Option<*mut Header>,
}

// Header
// Padding
// Back offset
// Data

// 2
//
// 100
// H (132)
// Data
// 8 geldi
// 132 + 32 = 164 % 8 != 0
// 4 byte bos
// H (190)
// Data
// o

impl KernelAllocator for LinkedListAllocator {
    unsafe fn new(start_addr: usize, end_addr: usize) -> Result<Self, ()> {
        // The allocator should at least be able to fit a single header
        if start_addr.checked_add(size_of::<Header>()).ok_or(())? > end_addr {
            return Err(());
        }

        let head = unsafe { NonNull::new(start_addr as *mut Header).ok_or(())?.as_mut() };
        head.sz = end_addr - start_addr;
        head.free = true;
        head.next = None;
        Ok(Self {
            start_addr,
            end_addr,
            head,
        })
    }
}

unsafe impl GlobalAlloc for LinkedListAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut cur_node_ptr = self.head;

        loop {
            let cur_node = unsafe { cur_node_ptr.as_mut().unwrap() };

            let header_start = cur_node_ptr as usize;
            let data_start = crate::align_up(
                header_start + size_of::<Header>() + size_of::<usize>(),
                layout.align(),
            );
            let back_offset_pos = data_start - size_of::<usize>();
            let back_offset = data_start - header_start;

            let alloc_size = back_offset + layout.size();

            if cur_node.free && cur_node.sz > alloc_size {
                let prev_sz = cur_node.sz;
                cur_node.sz = alloc_size;
                cur_node.free = false;

                unsafe {
                    *(back_offset_pos as *mut usize) = back_offset;
                }
                let remaining = prev_sz - alloc_size;

                if remaining >= (size_of::<Header>() + size_of::<usize>()) {
                    let next_node_ptr = unsafe { (cur_node as *mut Header).byte_add(cur_node.sz) };

                    unsafe {
                        *next_node_ptr = Header {
                            sz: remaining,
                            free: true,
                            next: cur_node.next,
                        };
                    }

                    cur_node.next = Some(next_node_ptr);
                } else {
                    cur_node.sz = prev_sz;
                }

                return data_start as *mut u8;
            }

            cur_node_ptr = match cur_node.next {
                Some(next) => next,
                None => panic!("kernel ran out of memory"),
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let back_offset = unsafe {
            let back_offset_ptr = ptr.byte_sub(size_of::<usize>()).cast::<usize>();
            if back_offset_ptr.is_null() {
                return;
            }

            *back_offset_ptr
        };

        let header = unsafe {
            match ptr.byte_sub(back_offset).cast::<Header>().as_mut() {
                Some(ptr) => ptr,
                None => return,
            }
        };

        header.free = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malloc_free() {
        let reserved_space = [0; 0x10000];

        let start_addr = reserved_space.as_ptr() as usize;

        let mut allocator = unsafe {
            LinkedListAllocator::new(start_addr, start_addr + reserved_space.len()).unwrap()
        };

        // [ (H || 64) || .. ]
        let a1 = allocator.allocate::<[u8; 64]>().unwrap();
        assert_eq!(a1.as_ptr() as usize, start_addr + size_of::<Header>());

        // [ (H || 64) || (H || 128) || .. ]
        let a2 = allocator.allocate::<[u8; 128]>().unwrap();
        assert_eq!(
            a2.as_ptr() as usize,
            a1.as_ptr() as usize + 64 + size_of::<Header>()
        );

        // [ (H || 64) || (H || 128) || (H || 192) || .. ]
        let a3 = allocator.allocate::<[u8; 192]>().unwrap();
        assert_eq!(
            a3.as_ptr() as usize,
            a2.as_ptr() as usize + 128 + size_of::<Header>()
        );

        // [ (H || 64) || (H || Free(128)) || (H || 192) || .. ]
        allocator.free(a2).unwrap();

        // [ (H || 64) || (H || 24) || (H || Free(104)) || (H || 192) || .. ]
        let a5 = allocator.allocate::<[u8; 24]>().unwrap();
        assert_eq!(
            a5.as_ptr() as usize,
            a1.as_ptr() as usize + 64 + size_of::<Header>()
        );

        // [ (H || 64) || (H || 24) || || (H || 32) || (H || Free(72)) || (H || 192) || .. ]
        let a6 = allocator.allocate::<[u8; 32]>().unwrap();
        assert_eq!(
            a6.as_ptr() as usize,
            a5.as_ptr() as usize + 24 + size_of::<Header>()
        );

        // [ (H || 64) || (H || 24) || || (H || 32) || (H || Free(72)) || (H || 192) || (H || 96) ]
        let a7 = allocator.allocate::<[u8; 96]>().unwrap();
        assert_eq!(
            a7.as_ptr() as usize,
            a3.as_ptr() as usize + 192 + size_of::<Header>()
        );
    }
}
