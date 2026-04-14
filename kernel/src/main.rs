#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![allow(unused)]

#[cfg(feature = "riscv-sbi")]
pub type Arch = arch::Riscv;

mod arch;
mod debug;
mod mm;

pub use debug::*;

core::arch::global_asm!(include_str!("start.s"));

#[unsafe(no_mangle)]
extern "C" fn kmain(_hartid: usize, _dtb_address: usize) {
    panic!();
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
