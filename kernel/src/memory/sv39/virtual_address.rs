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
    pub const BITS: u64 = 38;
    pub const MAX: u64 = (1 << Self::BITS) - 1;

    const MASK: u64 = 0b111111111;

    #[must_use]
    pub fn from_raw(addr: u64) -> Result<Self, ()> {
        if addr > Self::MAX {
            return Err(());
        }

        Ok(VirtualAddress(addr))
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
}
