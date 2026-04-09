mod boot;
mod context;
pub mod mmu;
mod trap;

use context::Context;
use riscv::registers::Satp;

use crate::arch::{
    Architecture, MemoryModel, PhysicalAddressOf, VirtualAddressOf,
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

    fn switch_to_user(
        from: *mut Self::Context,
        to: *const Self::Context,
        root_pt: PhysicalAddressOf<Self>,
    ) {
        unsafe {
            context::swtch_to_user(from, to, mmu::pa_to_satp(root_pt));
        }
    }

    fn set_kernel_sp(value: usize) {
        riscv::registers::Sscratch::new(value as u64).write();
    }

    fn trap_resume_ptr() -> *const () {
        trap_resume as *const ()
    }

    fn enable_interrupts() {
        riscv::registers::Sstatus::read()
            .enable_supervisor_interrupts()
            .write();

        riscv::registers::Sie::empty()
            .enable_external_interrupts()
            .enable_timer_interrupt()
            .write();
    }

    fn set_trap_handler(handler: usize) {
        riscv::registers::Stvec::new(handler as u64).write();
    }

    fn start_usermode(entry: VirtualAddressOf<Self>, user_sp: VirtualAddressOf<Self>) -> ! {
        riscv::registers::Sstatus::read()
            .enable_user_mode()
            .enable_user_page_access()
            .write();

        riscv::registers::Sepc::new(entry.raw()).write();

        riscv::sret(user_sp.raw());
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
