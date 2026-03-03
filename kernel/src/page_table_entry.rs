pub enum Flag {
    V = 0,
    R = 1,
    W = 1 << 1,
    X = 1 << 2,
    U = 1 << 3,
    G = 1 << 4,
    A = 1 << 5,
    D = 1 << 7,
}

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

    pub fn physical_address(&self) -> u64 {
        self.0 & Self::MASK_PPN
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
    pub fn set_physical_address(mut self, addr: u64) -> Self {
        debug_assert!(addr & 4096 == 0, "Physical address must be 4K-aligned");
        let offset_stripped = addr >> 12;
        self.0 =
            (!Self::MASK_PPN & self.0) | ((offset_stripped << Self::OFFSET_PPN_0) & Self::MASK_PPN);
        self
    }
}
