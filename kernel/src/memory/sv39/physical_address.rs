use crate::memory::virtual_address::VirtualAddress;

/// Physical address
///
/// Physical address is 55 bits wide and the layout is as follows:
///
///  55          30 29        21 20        12 11        0
/// +--------------+------------+------------+-----------+
/// |    PPN\[2\]  |   PPN\[1\] |   PPN\[0\] |   offset  |
/// +--------------+------------+------------+-----------+
///        26             9            9          12
///
/// https://docs.riscv.org/reference/isa/priv/supervisor.html#addressing-and-memory-protection
#[derive(Copy, Clone)]
pub struct PhysicalAddress(u64);

impl PhysicalAddress {
    pub const BITS: u64 = 55;
    pub const MAX: u64 = (1 << Self::BITS) - 1;
    pub const ZERO: PhysicalAddress = PhysicalAddress(0);

    #[must_use]
    pub fn from_raw(addr: u64) -> Result<PhysicalAddress, ()> {
        if addr > Self::MAX {
            return Err(());
        }

        Ok(PhysicalAddress(addr))
    }

    /// Safety:
    /// - `addr` must be at most `Self::MAX`
    #[must_use]
    pub unsafe fn from_raw_unchecked(addr: u64) -> PhysicalAddress {
        debug_assert!(addr <= Self::MAX);

        PhysicalAddress(addr)
    }

    /// Returns `true` if the physical address is page(4K) aligned.
    #[must_use]
    pub const fn is_page_aligned(&self) -> bool {
        self.0 & 0b111111111111 == self.0
    }

    #[must_use]
    pub const fn raw(&self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    #[must_use]
    pub const fn as_ptr_mut<T>(&self) -> *mut T {
        self.0 as *mut T
    }

    /// Creates a virtual address such that 0xABC as VA maps to 0xABC.
    ///
    /// Returns error if PA > VA::MAX since in that case, the VA cannot be
    /// identity mapped.
    #[must_use]
    pub fn to_identity_mapped_va(self) -> Result<VirtualAddress, ()> {
        VirtualAddress::from_raw(self.0)
    }
}
