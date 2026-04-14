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
pub struct Sie(usize);

impl Sie {
    pub const LCOFIE_SHIFT: usize = 13;
    pub const LCOFIE_MASK: usize = 1 << Self::LCOFIE_SHIFT;

    pub const SEIE_SHIFT: usize = 9;
    pub const SEIE_MASK: usize = 1 << Self::SEIE_SHIFT;

    pub const STIE_SHIFT: usize = 5;
    pub const STIE_MASK: usize = 1 << Self::STIE_SHIFT;

    pub const SSIE_SHIFT: usize = 1;
    pub const SSIE_MASK: usize = 1 << Self::SSIE_SHIFT;

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
