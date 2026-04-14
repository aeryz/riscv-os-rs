#![allow(unused)]

#[cfg(feature = "riscv-sbi")]
mod riscv;

#[cfg(feature = "riscv-sbi")]
pub use riscv::*;

/// Defines all the architecture-dependent functionality.
pub trait Architecture {
    const CPU_HERTZ: usize;

    type MemoryModel: MemoryModel;
}

pub type VirtualAddressOf<Arch> =
    <<Arch as Architecture>::MemoryModel as MemoryModel>::VirtualAddress;
pub type PhysicalAddressOf<Arch> =
    <<Arch as Architecture>::MemoryModel as MemoryModel>::PhysicalAddress;

pub trait MemoryModel {
    type PhysicalAddress: Into<usize>;

    type VirtualAddress: Into<usize>;

    fn set_root_page_table(pa: Self::PhysicalAddress);

    fn get_root_page_table() -> usize;
}
