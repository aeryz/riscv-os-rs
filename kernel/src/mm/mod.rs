mod allocator;
mod kvm;
mod mappings;

pub use allocator::{alloc, free};
pub use kvm::*;
pub use mappings::*;

use crate::{
    Arch,
    arch::{PhysicalAddressOf, VirtualAddressOf, mmu::PhysicalAddress},
};

pub const MAX_REGIONS: usize = 16;

pub const ADDRESS_SPACE_EMPTY: AddressSpace = AddressSpace {
    root_pt: PhysicalAddress::ZERO,
    regions: [const { None }; MAX_REGIONS],
};

#[derive(Clone)]
pub struct VmRegion {
    pub start: VirtualAddressOf<Arch>,
    pub end: VirtualAddressOf<Arch>,
    // TODO(aeryz): Add flags
}

#[derive(Clone)]
pub struct AddressSpace {
    pub root_pt: PhysicalAddressOf<Arch>,
    pub regions: [Option<VmRegion>; MAX_REGIONS],
}
