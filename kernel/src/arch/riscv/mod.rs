mod boot;
mod context;
pub mod mmu;
pub mod plic;
pub mod trap;

use core::ptr::NonNull;

use riscv::registers::Satp;

use crate::arch::{
    Architecture, MemoryModel, PhysicalAddressOf, VirtualAddressOf,
    mmu::{PageTable, PhysicalAddress, VirtualAddress},
    trap::{
        trap::{trap_entry, trap_resume},
        trap_frame::TrapFrame,
    },
};

use context::Context;

pub struct Riscv;

impl Architecture for Riscv {
    const CPU_HERTZ: usize = 10_000_000;

    type MemoryModel = Self;

    type TrapFrame = TrapFrame;

    type Context = Context;

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

    fn init_trap_handler() {
        riscv::registers::Stvec::new(trap_entry as *const () as usize).write();
    }

    fn enable_interrupts() {
        riscv::registers::Sstatus::read()
            .enable_supervisor_interrupts()
            .write();

        riscv::registers::Sie::empty()
            .enable_external_interrupts()
            // .enable_timer_interrupt()
            .write();
    }

    fn init_uart(core_id: usize) {
        plic::plic_init_uart(core_id);
    }

    fn switch_to(from: *mut Self::Context, to: *const Self::Context) {
        unsafe { context::swtch(from, to) };
    }

    fn switch_to_user(
        from: *mut Self::Context,
        to: *const Self::Context,
        root_pt: PhysicalAddressOf<Self>,
    ) {
        unsafe { context::swtch_to_user(from, to, mmu::pa_to_satp(root_pt)) };
    }

    fn set_per_cpu_ctx_ptr(ptr: VirtualAddressOf<Self>) {
        unsafe {
            core::arch::asm!(
                "mv tp, {}",
                in(reg) ptr.raw()
            );
        }
    }

    fn trap_resume_ptr() -> VirtualAddressOf<Self> {
        VirtualAddress::from_raw(trap_resume as *const () as usize).unwrap()
    }

    fn setup_unpriviledged_mode() {
        riscv::registers::Sstatus::read()
            .enable_user_mode()
            .enable_user_page_access()
            .write();
    }

    fn set_kernel_sp(sp: Option<VirtualAddressOf<Self>>) {
        riscv::registers::Sscratch::new(match sp {
            None => 0,
            Some(sp) => sp.raw(),
        })
        .write();
    }

    fn set_timer(time_val: usize) {
        riscv::sbi::set_timer(time_val);
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
