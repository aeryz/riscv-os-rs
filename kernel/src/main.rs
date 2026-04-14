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

use crate::arch::MemoryModel;

core::arch::global_asm!(include_str!("start.s"));

core::arch::global_asm!(
    r#"
    .section .text.harts
    .globl hart2_trampoline

hart2_trampoline:
    mv sp, a1
    call hart2
    "#,
);

unsafe extern "C" {
    pub fn hart2_trampoline() -> !;
}

#[unsafe(no_mangle)]
extern "C" fn kmain(_hartid: usize, _dtb_address: usize) {
    kdebug("hello from kernel\n");

    let hart2_addr = hart2_trampoline as *const () as usize
        - (mm::KERNEL_IMAGE_START_VA.raw() - mm::KERNEL_IMAGE_START_PA.raw());
    let hart3_addr = hart3 as *const () as usize
        - (mm::KERNEL_IMAGE_START_VA.raw() - mm::KERNEL_IMAGE_START_PA.raw());
    let hart4_addr = hart4 as *const () as usize
        - (mm::KERNEL_IMAGE_START_VA.raw() - mm::KERNEL_IMAGE_START_PA.raw());

    kdebug(usize_to_str_hex(hart2_addr, &mut [0; 20]));

    let buf1 = [0u8; 120];
    let ret = riscv::sbi::hart_start(
        1,
        hart2_addr,
        &buf1 as *const u8 as usize
            - (mm::KERNEL_IMAGE_START_VA.raw() - mm::KERNEL_IMAGE_START_PA.raw()),
    );
    if ret.error == 0 {
        kdebug("hart started successfully\n");
    } else {
        kdebug("hart start failure\n");
        kdebug(usize_to_str((-ret.error) as usize, &mut [0; 32]));
    }

    let buf2 = [0u8; 120];
    let ret = riscv::sbi::hart_start(
        2,
        hart3_addr,
        &buf2 as *const u8 as usize
            - (mm::KERNEL_IMAGE_START_VA.raw() - mm::KERNEL_IMAGE_START_PA.raw()),
    );
    if ret.error == 0 {
        kdebug("hart started successfully\n");
    } else {
        kdebug("hart start failure\n");
        kdebug(usize_to_str((-ret.error) as usize, &mut [0; 32]));
    }

    let buf3 = [0u8; 120];
    let ret = riscv::sbi::hart_start(
        3,
        hart4_addr,
        &buf2 as *const u8 as usize
            - (mm::KERNEL_IMAGE_START_VA.raw() - mm::KERNEL_IMAGE_START_PA.raw()),
    );
    if ret.error == 0 {
        kdebug("hart started successfully\n");
    } else {
        kdebug("hart start failure\n");
        kdebug(usize_to_str((-ret.error) as usize, &mut [0; 32]));
    }

    loop {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
extern "C" fn hart2(_hart_id: usize, _sp: usize) -> ! {
    kdebug("hello from hart2\n");
    loop {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
extern "C" fn hart3() -> ! {
    kdebug("hello from hart3\n");
    loop {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
extern "C" fn hart4() -> ! {
    kdebug("hello from hart4\n");
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
