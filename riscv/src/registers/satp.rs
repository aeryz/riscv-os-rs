use crate::impl_control_register;

/// `satp` register
///
/// `satp` register layout on 64-bits is as follows:
///
///  63          60 59        44 43          0
/// +--------------+------------+-------------+
/// | MODE (WARL)  | ASID (WARL)|  PPN (WARL) |
/// +--------------+------------+-------------+
///        4            16            44      
///
/// https://docs.riscv.org/reference/isa/priv/supervisor.html#satp
#[repr(transparent)]
pub struct Satp(u64);

impl Satp {
    pub const MODE_SHIFT: u64 = 60;
    pub const MODE_MASK: u64 = 0b1111 << Self::MODE_SHIFT;

    pub const PPN_MASK: u64 = (1 << 44) - 1;

    #[must_use]
    pub const fn set_mode(mut self, mode: SatpMode) -> Self {
        self.0 = (self.0 & (!Self::MODE_MASK)) | ((mode as u64) << Self::MODE_SHIFT);
        self
    }

    #[must_use]
    pub const fn set_ppn(mut self, ppn: u64) -> Self {
        self.0 = (self.0 & (!Self::PPN_MASK)) | ((ppn >> 12) & Self::PPN_MASK);
        self
    }
}

#[repr(u64)]
pub enum SatpMode {
    Bare = 0,
    Sv39 = 8,
    Sv48 = 9,
    Sv57 = 10,
    Sv64 = 11,
}

impl_control_register!(Satp, satp);
