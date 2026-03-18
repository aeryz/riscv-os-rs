use crate::{impl_bit_set, impl_control_register};

#[repr(transparent)]
pub struct Menvcfg(u64);

impl Menvcfg {
    pub const STCE_SHIFT: u64 = 63;
    pub const STCE_MASK: u64 = 1 << Self::STCE_SHIFT;

    #[must_use]
    pub fn enable_stimecmp(self) -> Self {
        self.set_stce()
    }

    impl_bit_set!(set_stce, STCE_MASK);
}

impl_control_register!(Menvcfg, menvcfg);
