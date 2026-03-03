use crate::memory::physical_address::PhysicalAddress;

/// Sv39 Page Table Entry (PTE) Layout
///
/// A Sv39 page table entry is 64 bits wide and structured as follows:
///
///  63  62  61 60     54 53      28 27      19 18      10 9   8  7   6   5   4   3   2   1   0
/// +---+------+---------+----------+----------+----------+-----+---+---+---+---+---+---+---+---+
/// | N | PBMT | Reserved|  PPN[2]  |  PPN[1]  |  PPN[0]  | RSW | D | A | G | U | X | W | R | V |
/// +---+------+---------+----------+----------+----------+-----+---+---+---+---+---+---+---+---+
///   1    2        7         26         9          9        2    1   1   1   1   1   1   1   1
///
/// https://docs.riscv.org/reference/isa/priv/supervisor.html#addressing-and-memory-protection
#[derive(Copy, Clone)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    const FLAG_V: u64 = 1;
    const FLAG_R: u64 = 1 << 1;
    const FLAG_W: u64 = 1 << 2;
    const FLAG_X: u64 = 1 << 3;
    const FLAG_U: u64 = 1 << 4;
    const FLAG_G: u64 = 1 << 5;
    const FLAG_A: u64 = 1 << 6;
    const FLAG_D: u64 = 1 << 7;

    // const MASK_PPN_0: u64 = 0x1ff << Self::OFFSET_PPN_0;
    // const MASK_PPN_1: u64 = 0x1ff << Self::OFFSET_PPN_1;
    // const MASK_PPN_2: u64 = 0x3ffffff << Self::OFFSET_PPN_2;

    const MASK_PPN: u64 = ((1u64 << 44) - 1) << Self::OFFSET_PPN_0;

    const OFFSET_PPN_0: u64 = 10;
    // const OFFSET_PPN_1: u64 = 19;
    // const OFFSET_PPN_2: u64 = 28;

    /// Returns a non-leaf valid PTE.
    #[must_use]
    pub fn new_pointer() -> Self {
        Self::empty().set_valid()
    }

    #[must_use]
    pub fn empty() -> Self {
        Self(0)
    }

    pub fn physical_address(&self) -> PhysicalAddress {
        unsafe { PhysicalAddress::from_raw_unchecked((self.0 & Self::MASK_PPN) << 2) }
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.0 & Self::FLAG_V == 1
    }

    #[must_use]
    pub fn set_valid(mut self) -> Self {
        self.0 |= Self::FLAG_V;
        self
    }

    #[must_use]
    pub fn set_readable(mut self) -> Self {
        self.0 |= Self::FLAG_R;
        self
    }

    #[must_use]
    pub fn set_writable(mut self) -> Self {
        self.0 |= Self::FLAG_W | Self::FLAG_R;
        self
    }

    #[must_use]
    pub fn set_executable(mut self) -> Self {
        self.0 |= Self::FLAG_X | Self::FLAG_R;
        self
    }

    #[must_use]
    pub fn set_rwx(mut self) -> Self {
        self.0 |= Self::FLAG_R | Self::FLAG_W | Self::FLAG_X;
        self
    }

    #[must_use]
    pub fn set_user_accessible(mut self) -> Self {
        self.0 |= Self::FLAG_U;
        self
    }

    #[must_use]
    pub fn set_global_mapping(mut self) -> Self {
        self.0 |= Self::FLAG_G;
        self
    }

    #[must_use]
    pub fn set_accessed(mut self) -> Self {
        self.0 |= Self::FLAG_A;
        self
    }

    #[must_use]
    pub fn set_dirty(mut self) -> Self {
        self.0 |= Self::FLAG_D;
        self
    }

    /// Sets the PPN\[2\], PPN\[1\] and PPN\[0\] to the bits[55:12] of the given `addr`
    #[must_use]
    pub fn set_physical_address(mut self, addr: PhysicalAddress) -> Self {
        debug_assert!(
            addr.is_page_aligned(),
            "Physical address must be 4K-aligned"
        );
        let offset_stripped = addr.raw() >> 12;
        self.0 =
            (!Self::MASK_PPN & self.0) | ((offset_stripped << Self::OFFSET_PPN_0) & Self::MASK_PPN);
        self
    }
}
