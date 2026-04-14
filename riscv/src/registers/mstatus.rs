use crate::impl_control_register;

#[repr(transparent)]
pub struct Mstatus(usize);

impl Mstatus {
    pub const MPP_SHIFT: usize = 11;
    pub const MPP_MASK: usize = 0b11 << Self::MPP_SHIFT;

    pub const SIE_SHIFT: usize = 1;
    pub const SIE_MASK: usize = 1 << Self::SIE_SHIFT;

    #[must_use]
    pub fn enable_supervisor_mode(self) -> Self {
        self.set_mpp(MstatusMpp::S)
    }

    #[must_use]
    pub fn set_sie(mut self) -> Self {
        self.0 |= Self::SIE_MASK;
        self
    }

    #[must_use]
    pub fn set_mpp(mut self, spp: MstatusMpp) -> Self {
        self.0 = (self.0 & (!Self::MPP_MASK)) | ((spp as usize) << Self::MPP_SHIFT);
        self
    }
}

#[repr(usize)]
pub enum MstatusMpp {
    U = 0b00,
    S = 0b01,
    M = 0b11,
}

impl_control_register!(Mstatus, mstatus);
