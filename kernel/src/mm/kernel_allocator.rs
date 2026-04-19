use ksync::SpinLock;

use crate::{
    Arch,
    arch::{VirtualAddressOf, mmu::VirtualAddress},
};

static ALLOCATOR: SpinLock<Allocator> = SpinLock::new(Allocator {
    start_addr: unsafe { VirtualAddress::from_raw_unchecked(0) },
    end_addr: unsafe { VirtualAddress::from_raw_unchecked(0) },
    root: core::ptr::null_mut(),
});

#[repr(C)]
struct Allocator {
    start_addr: VirtualAddressOf<Arch>,
    end_addr: VirtualAddressOf<Arch>,
    root: *mut AllocatorNode,
}

struct AllocatorNode {
    sz: usize,
    free: bool,
    next: Option<*mut AllocatorNode>,
}

pub fn init(start_addr: VirtualAddressOf<Arch>, end_addr: VirtualAddressOf<Arch>) {
    let mut allocator = ALLOCATOR.lock();
    allocator.start_addr = start_addr;
    allocator.end_addr = end_addr;

    allocator.root = start_addr.as_ptr_mut();

    let allocator_node = unsafe { allocator.root.as_mut().unwrap() };

    allocator_node.sz = end_addr.raw() - start_addr.raw();
    allocator_node.free = true;
}

pub fn kmalloc(sz: usize) -> VirtualAddressOf<Arch> {
    let allocator = ALLOCATOR.lock();
    let mut cur_node_ptr = allocator.root;
    loop {
        let cur_node = unsafe { cur_node_ptr.as_mut().unwrap() };
        // TODO(aeryz): need to mind alignment as well
        if cur_node.free && cur_node.sz > (sz + size_of::<AllocatorNode>()) {
            // we found it mate
            let prev_sz = cur_node.sz;
            cur_node.sz = sz + size_of::<AllocatorNode>();
            cur_node.free = false;

            let next_node_ptr =
                unsafe { (cur_node as *mut AllocatorNode).byte_offset(cur_node.sz as isize) };
            unsafe {
                *next_node_ptr = AllocatorNode {
                    sz: prev_sz - cur_node.sz,
                    free: true,
                    next: cur_node.next,
                };
            }

            cur_node.next = Some(next_node_ptr);

            return VirtualAddress::from_raw(cur_node_ptr as usize + size_of::<AllocatorNode>())
                .unwrap();
        }

        cur_node_ptr = match cur_node.next {
            Some(next) => next,
            None => panic!("kernel ran out of memory"),
        }
    }
}

pub fn kfree(ptr: VirtualAddressOf<Arch>) {
    let allocator_node = unsafe {
        ((ptr.raw() - size_of::<AllocatorNode>()) as *mut AllocatorNode)
            .as_mut()
            .unwrap()
    };

    allocator_node.free = false;
}

/// SAFETY:
/// - Allocator is only accessed and modified through `SpinLock`.
unsafe impl Send for Allocator {}
