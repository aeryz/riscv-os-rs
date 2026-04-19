//! ## Linked list allocator

use core::ptr::NonNull;

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

impl LinkedListAllocator {
    /// Creates a new allocator
    ///
    /// * `start_addr`: The start of the address that is reserved for this allocator.
    /// * `end_addr`: The end of the address that is reserved for this allocator.
    ///
    /// ## Safety
    /// - `start_addr` and `end_addr` are valid addresses during the execution of this allocator.
    /// It's a really common mistake to initialize this allocator with physical addresses before
    /// starting paging and then immediately get a trap once paging is enabled.
    pub unsafe fn new(start_addr: usize, end_addr: usize) -> Result<Self, ()> {
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

    /// Allocate `size_of::<T>()` bytes
    pub fn alloc<T>(&mut self) -> Result<NonNull<T>, ()> {
        let mut cur_node_ptr = self.head;

        let alloc_size = size_of::<T>().checked_add(size_of::<Header>()).ok_or(())?;

        loop {
            let cur_node = unsafe { cur_node_ptr.as_mut().unwrap() };
            if cur_node.free && cur_node.sz > alloc_size {
                let prev_sz = cur_node.sz;
                cur_node.sz = alloc_size;
                cur_node.free = false;

                let next_node_ptr = unsafe { (cur_node as *mut Header).byte_add(cur_node.sz) };
                unsafe {
                    *next_node_ptr = Header {
                        sz: prev_sz - cur_node.sz,
                        free: true,
                        next: cur_node.next,
                    };
                }

                cur_node.next = Some(next_node_ptr);

                return Ok(unsafe {
                    NonNull::new_unchecked(cur_node_ptr.byte_add(size_of::<Header>()) as *mut T)
                });
            }

            cur_node_ptr = match cur_node.next {
                Some(next) => next,
                None => panic!("kernel ran out of memory"),
            }
        }
    }

    /// Frees the allocated memoty for `ptr`.
    ///
    /// Note that this only labels the memory as freed. It does not zero out the storage.
    ///
    /// - `ptr`: The pointer to the allocated space initially acquired using [`alloc`];
    pub fn free<T>(&mut self, ptr: NonNull<T>) -> Result<(), ()> {
        let header = unsafe { ptr.byte_sub(size_of::<Header>()).cast::<Header>().as_mut() };
        header.free = true;
        Ok(())
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
        let a1 = allocator.alloc::<[u8; 64]>().unwrap();
        assert_eq!(a1.as_ptr() as usize, start_addr + size_of::<Header>());

        // [ (H || 64) || (H || 128) || .. ]
        let a2 = allocator.alloc::<[u8; 128]>().unwrap();
        assert_eq!(
            a2.as_ptr() as usize,
            a1.as_ptr() as usize + 64 + size_of::<Header>()
        );

        // [ (H || 64) || (H || 128) || (H || 192) || .. ]
        let a3 = allocator.alloc::<[u8; 192]>().unwrap();
        assert_eq!(
            a3.as_ptr() as usize,
            a2.as_ptr() as usize + 128 + size_of::<Header>()
        );

        // [ (H || 64) || (H || Free(128)) || (H || 192) || .. ]
        allocator.free(a2).unwrap();

        // [ (H || 64) || (H || 24) || (H || Free(104)) || (H || 192) || .. ]
        let a5 = allocator.alloc::<[u8; 24]>().unwrap();
        assert_eq!(
            a5.as_ptr() as usize,
            a1.as_ptr() as usize + 64 + size_of::<Header>()
        );

        // [ (H || 64) || (H || 24) || || (H || 32) || (H || Free(72)) || (H || 192) || .. ]
        let a6 = allocator.alloc::<[u8; 32]>().unwrap();
        assert_eq!(
            a6.as_ptr() as usize,
            a5.as_ptr() as usize + 24 + size_of::<Header>()
        );

        // [ (H || 64) || (H || 24) || || (H || 32) || (H || Free(72)) || (H || 192) || (H || 96) ]
        let a7 = allocator.alloc::<[u8; 96]>().unwrap();
        assert_eq!(
            a7.as_ptr() as usize,
            a3.as_ptr() as usize + 192 + size_of::<Header>()
        );
    }
}
