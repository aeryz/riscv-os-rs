use crate::{impl_bit_set, impl_control_register};

/// `sie` register
///
/// ## Layout
///
/// 15  14  13   12  10  9  8   6   5  4   2   1    0
/// +---+--------+---+------+---+------+---+------+---+
/// | 0 | LCOFIE | 0 | SEIE | 0 | STIE | 0 | SSIE | 0 |
/// +---+--------+---+------+---+------+---+------+---+
///   2      1     3     1    3     1    3    1     1     
///
/// https://docs.riscv.org/reference/isa/priv/supervisor.html#satp
#[repr(transparent)]
pub struct Sie(u64);

impl Sie {
    pub const LCOFIE_SHIFT: u64 = 13;
    pub const LCOFIE_MASK: u64 = 1 << Self::LCOFIE_SHIFT;

    pub const SEIE_SHIFT: u64 = 9;
    pub const SEIE_MASK: u64 = 1 << Self::SEIE_SHIFT;

    pub const STIE_SHIFT: u64 = 5;
    pub const STIE_MASK: u64 = 1 << Self::STIE_SHIFT;

    pub const SSIE_SHIFT: u64 = 1;
    pub const SSIE_MASK: u64 = 1 << Self::SSIE_SHIFT;

    #[must_use]
    pub fn enable_external_interrupts(self) -> Self {
        self.set_seie()
    }

    #[must_use]
    pub fn enable_timer_interrupt(self) -> Self {
        self.set_stie()
    }

    impl_bit_set!(set_lcofie, LCOFIE_MASK);
    impl_bit_set!(set_seie, SEIE_MASK);
    impl_bit_set!(set_stie, STIE_MASK);
    impl_bit_set!(set_ssie, SSIE_MASK);
}

impl_control_register!(Sie, sie);
