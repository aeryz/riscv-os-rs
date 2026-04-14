mod boot;
pub mod mmu;

use riscv::registers::Satp;

use crate::arch::{
    Architecture, MemoryModel, VirtualAddressOf,
    mmu::{PhysicalAddress, VirtualAddress},
};

pub struct Riscv;

impl Architecture for Riscv {
    const CPU_HERTZ: usize = 10_000_000;

    type MemoryModel = Self;
}

impl MemoryModel for Riscv {
    type PhysicalAddress = PhysicalAddress;

    type VirtualAddress = VirtualAddress;

    fn set_root_page_table(pa: Self::PhysicalAddress) {
        mmu::set_root_page_table(pa);
    }

    fn get_root_page_table() -> usize {
        Satp::read().raw() as usize
    }
}
