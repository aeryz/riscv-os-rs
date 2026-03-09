use crate::impl_control_register;

#[repr(transparent)]
pub struct Mstatus(u64);

impl Mstatus {
    pub const MPP_SHIFT: u64 = 11;
    pub const MPP_MASK: u64 = 0b11 << Self::MPP_SHIFT;

    pub const SIE_SHIFT: u64 = 1;
    pub const SIE_MASK: u64 = 1 << Self::SIE_SHIFT;

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
        self.0 = (self.0 & (!Self::MPP_MASK)) | ((spp as u64) << Self::MPP_SHIFT);
        self
    }
}

#[repr(u64)]
pub enum MstatusMpp {
    U = 0b00,
    S = 0b01,
    M = 0b11,
}

impl_control_register!(Mstatus, mstatus);
