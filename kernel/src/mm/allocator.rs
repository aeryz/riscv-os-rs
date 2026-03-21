use core::cell::OnceCell;

use crate::memory::physical_address::PhysicalAddress;

static mut ALLOCATOR: OnceCell<Allocator<16>> = OnceCell::new();

// The allocator public api:

/// Initialize the allocator with the given `start_addr`.
/// This will only `init` once even if it is called multiple times.
pub fn init(start_addr: PhysicalAddress) {
    unsafe {
        let _ = ALLOCATOR.get_or_init(|| Allocator::new(start_addr));
    }
}

// TODO: be able to allocate a custom amount (will probably have this while implementing sbrk)
/// Allocate a single 4k page.
pub fn alloc() -> Result<PhysicalAddress, ()> {
    unsafe { ALLOCATOR.get_mut().unwrap().alloc() }
}

/// Dumb allocator
///
/// Starting from the `start_addr`, support 64 * N pages.
///
/// This allocator only works with maximum of 4K alloc requests.
/// It doesn't have a smart mechanism to walk through the pages.
/// It just tries to find the next empty table by going through
/// the `pages` bitfields.
#[repr(C)]
struct Allocator<const N: usize> {
    start_addr: PhysicalAddress,
    pages: [u64; N],
}

impl<const N: usize> Allocator<N> {
    #[must_use]
    const fn new(start_addr: PhysicalAddress) -> Self {
        Self {
            start_addr,
            pages: [0; N],
        }
    }

    /// Allocates a single page and returns its start address
    ///
    /// Returns `Err` if there is no memory left.
    #[must_use]
    fn alloc(&mut self) -> Result<PhysicalAddress, ()> {
        for page_i in 0..self.pages.len() {
            for i in 0..u64::BITS {
                if (self.pages[page_i] >> i) & 1 == 0 {
                    self.pages[page_i] |= 1 << i;
                    let alloc_addr = (page_i as u64 * 64 + i as u64) * 4096;
                    // Safety:
                    // - TODO: we must guarantee that the allocator is properly configured s.t.
                    // we can't produce an out of bounds physical address
                    return Ok(unsafe {
                        PhysicalAddress::from_raw_unchecked(self.start_addr.raw() + alloc_addr)
                    });
                }
            }
        }

        Err(())
    }
}
