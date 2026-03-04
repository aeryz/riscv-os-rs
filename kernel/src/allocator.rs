use crate::memory::physical_address::PhysicalAddress;

/// Dumb allocator
///
/// Starting from the `start_addr`, support 64 * N pages.
///
/// This allocator only works with maximum of 4K alloc requests.
/// It doesn't have a smart mechanism to walk through the pages.
/// It just tries to find the next empty table by going through
/// the `pages` bitfields.
#[repr(C)]
pub struct Allocator<const N: usize> {
    start_addr: PhysicalAddress,
    pages: [u64; N],
}

impl<const N: usize> Allocator<N> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            start_addr: PhysicalAddress::ZERO,
            pages: [0; N],
        }
    }

    pub const fn set_start_addr(&mut self, start_addr: PhysicalAddress) {
        self.start_addr = start_addr;
    }

    /// Allocates a single page and returns its start address
    ///
    /// Returns `Err` if there is no memory left.
    #[must_use]
    #[inline(never)]
    pub fn alloc(&mut self) -> Result<PhysicalAddress, ()> {
        for page_i in 0..self.pages.len() {
            for i in 0..u64::BITS {
                if (self.pages[page_i] >> i) & 1 == 0 {
                    self.pages[page_i] |= 1 << i;
                    let alloc_addr = page_i as u64 * 64 + i as u64 * 4096;
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
