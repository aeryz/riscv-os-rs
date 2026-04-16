mod boot;
pub mod mmu;

use core::ptr::NonNull;

use riscv::registers::Satp;

use crate::arch::{
    Architecture, MemoryModel, VirtualAddressOf,
    mmu::{PageTable, PhysicalAddress, VirtualAddress},
};

pub struct Riscv;

impl Architecture for Riscv {
    const CPU_HERTZ: usize = 10_000_000;

    type MemoryModel = Self;

    fn bump_sp(sp: usize) {
        riscv::add_to_sp(sp);
    }

    fn load_this_cpu_ctx<T>() -> *mut T {
        riscv::read_tp() as *mut T
    }

    fn read_current_time() -> usize {
        // TODO(aeryz): through sbi
        todo!()
    }
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
