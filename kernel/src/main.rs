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

pub use debug::*;

use crate::{
    arch::{Architecture, MemoryModel},
    driver::uart,
};

core::arch::global_asm!(include_str!("start.s"));

#[unsafe(no_mangle)]
extern "C" fn kmain(hartid: usize, dtb_address: usize) {
    serial_log::init();
    log::info!("Kernel starts with hart_id: {hartid}, dtb: 0x{dtb_address:x}",);

    Arch::init_trap_handler();

    Arch::init_uart(hartid);

    uart::enable_interrupts();
    Arch::enable_interrupts();

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
