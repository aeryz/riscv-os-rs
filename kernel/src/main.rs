#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![allow(unused)]

#[cfg(feature = "riscv-sbi")]
pub type Arch = arch::Riscv;

mod arch;
mod debug;
mod mm;
mod percpu;
mod sched;
mod task;

pub use debug::*;

use crate::arch::MemoryModel;

core::arch::global_asm!(include_str!("start.s"));

#[unsafe(no_mangle)]
extern "C" fn kmain(_hartid: usize, _dtb_address: usize) {
    kdebug("hello from kernel\n");

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
