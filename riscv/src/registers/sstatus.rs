use crate::impl_control_register;

#[repr(transparent)]
pub struct Sstatus(usize);

impl Sstatus {
    pub const SUM_SHIFT: usize = 18;
    pub const SUM_MASK: usize = 1 << Self::SUM_SHIFT;

    pub const SPP_SHIFT: usize = 8;
    pub const SPP_MASK: usize = 1 << Self::SPP_SHIFT;

    pub const SIE_SHIFT: usize = 1;
    pub const SIE_MASK: usize = 1 << Self::SIE_SHIFT;

    pub const SPIE_SHIFT: usize = 5;
    pub const SPIE_MASK: usize = 1 << Self::SPIE_SHIFT;

    #[must_use]
    pub fn enable_user_page_access(self) -> Self {
        self.set_sum()
    }

    #[must_use]
    pub fn enable_user_mode(self) -> Self {
        self.set_spp(SstatusSpp::U)
    }

    #[must_use]
    pub fn enable_supervisor_interrupts(self) -> Self {
        self.set_spie().set_sie()
    }

    #[must_use]
    pub fn disable_supervisor_interrupts(self) -> Self {
        self.unset_sie()
    }

    #[must_use]
    pub fn set_spp(mut self, spp: SstatusSpp) -> Self {
        self.0 = (self.0 & (!Self::SPP_MASK)) | ((spp as usize) << Self::SPP_SHIFT);
        self
    }

    #[must_use]
    pub fn set_sie(mut self) -> Self {
        self.0 |= Self::SIE_MASK;
        self
    }

    #[must_use]
    pub fn set_spie(mut self) -> Self {
        self.0 |= Self::SPIE_MASK;
        self
    }

    #[must_use]
    pub fn unset_sie(mut self) -> Self {
        self.0 &= !Self::SIE_MASK;
        self
    }

    #[must_use]
    pub fn set_sum(mut self) -> Self {
        self.0 |= Self::SUM_MASK;
        self
    }
}

#[repr(usize)]
pub enum SstatusSpp {
    U = 0,
    S = 1,
}

impl_control_register!(Sstatus, sstatus);
