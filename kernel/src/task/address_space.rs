use crate::{
    Arch,
    arch::{PhysicalAddressOf, VirtualAddressOf, mmu::PhysicalAddress},
};

pub const MAX_REGIONS: usize = 16;

pub const ADDRESS_SPACE_EMPTY: AddressSpace = AddressSpace {
    root_pt: PhysicalAddress::ZERO,
    regions: heapless::Vec::new(),
};

#[derive(Clone)]
pub struct VmRegion {
    /// Start address of this region
    pub start: VirtualAddressOf<Arch>,
    /// End address of this region
    pub end: VirtualAddressOf<Arch>,
    // TODO(aeryz): temporary field that let's us tell whether the process own this address space or not.
    // Since we don't have a filesystem right now, the userspace programs live under the kernel binary and
    // for that reason, we cannot free the `text` section after exit.
    pub process_owned: bool,
}

#[derive(Clone)]
pub struct AddressSpace {
    pub root_pt: PhysicalAddressOf<Arch>,
    pub regions: heapless::Vec<VmRegion, MAX_REGIONS>,
}
