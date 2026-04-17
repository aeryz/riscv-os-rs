#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![allow(unused)]

#[cfg(feature = "riscv-sbi")]
pub type Arch = arch::Riscv;

mod arch;
mod debug;
mod driver;
mod mm;
mod percpu;
mod sched;
mod serial_log;
mod syscall;
mod task;
mod userspace;

use core::ptr::NonNull;

pub use debug::*;
use ksync::SpinLock;

use crate::{
    arch::{Architecture, MemoryModel, mmu::VirtualAddress},
    driver::uart,
};

core::arch::global_asm!(include_str!("start.s"));

#[unsafe(no_mangle)]
extern "C" fn kmain(hartid: usize, dtb_address: usize) {
    serial_log::init();
    log::info!("Kernel starts with hart_id: {hartid}, dtb: 0x{dtb_address:x}",);

    Arch::init_trap_handler();
    log::info!("trap handler initiated");

    Arch::init_uart(hartid);
    log::info!("uart initiated");

    uart::enable_interrupts();
    log::info!("uart interrupts enabled");

    let idle_task = task::create_kernel_task(
        VirtualAddress::from_raw(idle_task as *const () as usize).unwrap(),
    );
    log::info!("idle task created");

    let mut core_ctxs = heapless::Vec::new();
    core_ctxs.push(percpu::PerCoreContext {
        core_id: 0,
        scheduler: SpinLock::new(sched::init_per_core_scheduler()),
        currently_running_task: idle_task,
        idle_task,
    });
    percpu::set_core_ctxs(core_ctxs);
    log::info!("per cpu data is set");

    let task_1 = task::create_task(
        VirtualAddress::from_raw(userspace::userspace_sleep_print_loop as *const () as usize)
            .unwrap(),
    );
    log::info!("task 1 is created");
    let task_2 = task::create_task(
        VirtualAddress::from_raw(userspace::userspace_sleep_print_loop2 as *const () as usize)
            .unwrap(),
    );
    log::info!("task 2 is created");

    log::info!("Core state: {:#?}", percpu::get_core(0));

    Arch::set_per_cpu_ctx_ptr(
        VirtualAddress::from_raw(percpu::get_core(0) as *const percpu::PerCoreContext as usize)
            .unwrap(),
    );
    Arch::setup_unpriviledged_mode();

    Arch::enable_interrupts();

    sched::schedule();
    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
#[inline(never)]
extern "C" fn idle_task() {
    log::debug!("idle mode");

    loop {
        riscv::registers::Sstatus::read()
            .enable_supervisor_interrupts()
            .write();
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}
