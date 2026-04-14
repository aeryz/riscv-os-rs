use crate::impl_control_register;

#[repr(transparent)]
pub struct Pmpcfg<const N: usize>(usize);

impl<const N: usize> Pmpcfg<N> {
    pub const READ_SHIFT: usize = 0;
    pub const READ_MASK: usize = 1;

    pub const WRITE_SHIFT: usize = 1;
    pub const WRITE_MASK: usize = 1 << Self::WRITE_SHIFT;

    pub const EXECUTE_SHIFT: usize = 2;
    pub const EXECUTE_MASK: usize = 1 << Self::EXECUTE_SHIFT;

    pub const AMM_SHIFT: usize = 3;
    pub const AMM_MASK: usize = 1 << Self::AMM_SHIFT;

    #[must_use]
    pub fn enable_tor(self) -> Self {
        self.set_amm(PmpAmm::Tor)
    }

    #[must_use]
    pub fn set_readable(mut self) -> Self {
        self.0 |= Self::READ_MASK;
        self
    }

    #[must_use]
    pub fn set_writable(mut self) -> Self {
        self.0 |= Self::WRITE_MASK;
        self
    }

    #[must_use]
    pub fn set_executable(mut self) -> Self {
        self.0 |= Self::EXECUTE_MASK;
        self
    }

    #[must_use]
    pub fn set_amm(mut self, amm: PmpAmm) -> Self {
        self.0 = (self.0 & (!Self::AMM_MASK)) | ((amm as usize) << Self::AMM_SHIFT);
        self
    }
}

#[repr(usize)]
pub enum PmpAmm {
    Off = 0b00,
    Tor = 0b01,
    Na4 = 0b10,
    Napot = 0b11,
}

pub type Pmpcfg0 = Pmpcfg<0>;

impl_control_register!(Pmpcfg0, pmpcfg0);
