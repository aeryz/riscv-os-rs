#![no_std]
#![no_main]
#![allow(static_mut_refs)]

#[cfg(feature = "riscv-sbi")]
pub type Arch = arch::Riscv;

extern crate alloc;

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
mod vfs;

use alloc::vec::Vec;
pub use debug::*;
use ksync::SpinLock;

use crate::{
    arch::{Architecture, mmu::VirtualAddress},
    driver::{uart, virtio},
};

core::arch::global_asm!(include_str!("start.s"));

#[unsafe(no_mangle)]
extern "C" fn kmain(hartid: usize, dtb_address: usize) -> ! {
    serial_log::init();
    log::info!("Kernel starts with hart_id: {hartid}, dtb: 0x{dtb_address:x}",);

    let blk_device_base = virtio::find_virtio_blk().unwrap();
    log::info!("Found device id: {blk_device_base:x}");

    match virtio::block::init(blk_device_base) {
        Ok(_) => log::info!("driver initialized"),
        Err(_) => log::error!("driver initialization failed"),
    }

    let mut core_ctxs = Vec::new();

    setup_core(0, &mut core_ctxs);
    #[cfg(feature = "multi-core")]
    {
        setup_core(1, &mut core_ctxs);
        setup_core(2, &mut core_ctxs);
    }

    percpu::set_core_ctxs(core_ctxs);

    let _ = task::create_task(unsafe {
        VirtualAddress::from_raw_unchecked(
            userspace::userspace_sleep_print_loop_1 as *const () as usize,
        )
    });
    let _ = task::create_task(unsafe {
        VirtualAddress::from_raw_unchecked(
            userspace::userspace_sleep_print_loop_2 as *const () as usize,
        )
    });
    let _ = task::create_task(unsafe {
        VirtualAddress::from_raw_unchecked(
            userspace::userspace_sleep_print_loop_3 as *const () as usize,
        )
    });
    let _ = task::create_task(unsafe {
        VirtualAddress::from_raw_unchecked(
            userspace::userspace_sleep_print_loop_4 as *const () as usize,
        )
    });

    #[cfg(feature = "multi-core")]
    {
        Arch::boot_core(1);
        Arch::boot_core(2);
    }
    core_boot_entry(0);
}

fn setup_core(core_id: usize, core_ctxs: &mut Vec<percpu::PerCoreContext>) {
    let idle_task = task::create_kernel_task(
        VirtualAddress::from_raw(idle_task_main as *const () as usize).unwrap(),
    );

    core_ctxs.push(percpu::PerCoreContext {
        core_id,
        scheduler: SpinLock::new(sched::init_per_core_scheduler()),
        currently_running_task: idle_task,
        idle_task,
    });
}

#[unsafe(no_mangle)]
extern "C" fn core_boot_entry(core: usize) -> ! {
    Arch::init_trap_handler();
    log::trace!("trap handler initiated");

    Arch::init_uart(core);
    log::trace!("uart initiated");

    uart::enable_interrupts();
    log::trace!("uart interrupts enabled");

    Arch::set_per_cpu_ctx_ptr(
        VirtualAddress::from_raw(percpu::get_core(core) as *const percpu::PerCoreContext as usize)
            .unwrap(),
    );
    Arch::setup_unpriviledged_mode();

    let time = Arch::read_current_time();
    Arch::set_timer(time + Arch::nanos_to_ticks(32 * 1_000_000));

    Arch::enable_interrupts();

    sched::schedule();

    idle_task_main();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("KERNEL PANIC: {}", info.message());
    if let Some(loc) = info.location() {
        log::error!("-> File: {} at line: {}", loc.file(), loc.line());
    }

    loop {
        Arch::halt();
    }
}

#[unsafe(no_mangle)]
#[inline(never)]
extern "C" fn idle_task_main() -> ! {
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
