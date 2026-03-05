/// Virtual address
///
/// Virtual address is 38 bits wide and the layout is as follows:
///
/// ```text
///  38        30 29      21 20      12 11          0
/// +------------+----------+----------+-------------+
/// |    VPN[2]  |   VPN[1] |   VPN[0] | page offset |
/// +------------+----------+----------+-------------+
///       9           9          9           12
/// ```
///
/// https://docs.riscv.org/reference/isa/priv/supervisor.html#addressing-and-memory-protection
#[derive(Copy, Clone)]
pub struct VirtualAddress(u64);

impl VirtualAddress {
    pub const BITS: u64 = 39;

    const MASK: u64 = 0b111111111;

    #[must_use]
    pub fn from_raw(addr: u64) -> Result<Self, ()> {
        let sign = (addr >> (Self::BITS - 1)) & 1;
        let upper = addr >> Self::BITS;

        if (sign == 0 && upper == 0) || (sign == 1 && upper == (1 << 25) - 1) {
            Ok(VirtualAddress(addr))
        } else {
            Err(())
        }
    }

    #[must_use]
    pub const fn vpn_2(&self) -> usize {
        ((self.0 >> 30) & Self::MASK) as usize
    }

    #[must_use]
    pub const fn vpn_1(&self) -> usize {
        ((self.0 >> 21) & Self::MASK) as usize
    }

    #[must_use]
    pub const fn vpn_0(&self) -> usize {
        ((self.0 >> 12) & Self::MASK) as usize
    }

    #[must_use]
    pub const fn as_ptr_mut<T>(&self) -> *mut T {
        self.0 as *mut T
    }

    #[must_use]
    pub const fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    #[must_use]
    pub const fn raw(&self) -> u64 {
        self.0
    }
}
