mod context;
pub mod mmu;
mod trap;

use context::Context;

use crate::arch::{
    Architecture, MemoryModel,
    mmu::{PhysicalAddress, VirtualAddress},
    riscv::trap::trap::trap_resume,
};

pub struct Riscv;

impl Architecture for Riscv {
    const CPU_HERTZ: usize = 10_000_000;

    type Context = Context;
    type MemoryModel = Self;
    type TrapFrame = trap::trap_frame::TrapFrame;

    /// Sets the timer interrupt
    // TODO(aeryz): make the arch a trait
    fn set_timer(time: usize) {
        riscv::registers::Stimecmp::new(time as u64).write();
    }

    fn read_current_time() -> usize {
        riscv::registers::Time::read().raw() as usize
    }

    fn switch(from: *mut Self::Context, to: *const Self::Context) {
        unsafe {
            context::swtch(from, to);
        }
    }

    fn set_kernel_sp(value: usize) {
        riscv::registers::Sscratch::new(value as u64).write();
    }

    fn trap_resume_ptr() -> *const () {
        trap_resume as *const ()
    }
}

impl MemoryModel for Riscv {
    type PhysicalAddress = PhysicalAddress;

    type VirtualAddress = VirtualAddress;

    fn set_root_page_table(pa: Self::PhysicalAddress) {
        mmu::set_root_page_table(pa);
    }
}
