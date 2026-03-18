use crate::{impl_bit_set, impl_control_register};

/// `mcounteren` register
///
/// ## Layout
///
///    31       30     29   28             6   5     4      3      2    1    0
/// +-------+-------+-------+--------------+------+------+------+----+----+----+
/// | HPM31 | HMP30 | HPM29 |      ...     | HPM5 | HPM4 | HPM3 | IR | TM | CY |
/// +-------+-------+-------+--------------+------+------+------+----+----+----+
///     1       1       1          23          1      1      1    1     1    1
///
/// https://docs.riscv.org/reference/isa/priv/supervisor.html#satp
#[repr(transparent)]
pub struct Mcounteren(u64);

impl Mcounteren {
    pub const IR_SHIFT: u64 = 2;
    pub const IR_MASK: u64 = 1 << Self::IR_SHIFT;

    pub const TM_SHIFT: u64 = 1;
    pub const TM_MASK: u64 = 1 << Self::TM_SHIFT;

    pub const CY_SHIFT: u64 = 0;
    pub const CY_MASK: u64 = 1 << Self::CY_SHIFT;

    #[must_use]
    pub fn enable_access_to_instret(self) -> Self {
        self.set_ir()
    }

    #[must_use]
    pub fn enable_access_to_time(self) -> Self {
        self.set_tm()
    }

    #[must_use]
    pub fn enable_access_to_cycle(self) -> Self {
        self.set_cy()
    }

    impl_bit_set!(set_ir, IR_MASK);
    impl_bit_set!(set_tm, TM_MASK);
    impl_bit_set!(set_cy, CY_MASK);
}

impl_control_register!(Mcounteren, mcounteren);
