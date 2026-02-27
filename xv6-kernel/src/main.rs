#![no_std]
#![no_main]

use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};

global_asm!(include_str!("start.s"));

const UART_ADDR: *mut u8 = 0x10000000 as *mut u8;

const MSTATUS_MPP_SHIFT: usize = 11;
const MSTATUS_MPP_S: usize = 0b01 << MSTATUS_MPP_SHIFT;
const PMP_0_CFG: usize = 0b00001111;

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    b"hello world from kernel\n"
        .into_iter()
        .for_each(|b| unsafe { core::ptr::write_volatile(UART_ADDR, *b) });

    enter_supervisor(start as *const () as usize);

    loop {
        core::hint::spin_loop();
    }
}

#[inline(always)]
pub fn mret() {
    unsafe {
        asm!("mret", options(nomem, nostack, preserves_flags));
    }
}

#[inline(always)]
pub fn enter_supervisor(entry: usize) {
    unsafe {
        asm!(
            "csrw mepc, {entry}",

            "csrr t0, pmpaddr0",
            "csrr t0, mstatus",
            "li t1, {mpp_s}",
            // unset 12th bit for setting the MPP to 01(S mode)
            "or t0, t0, t1",
            "slli t1, t1, 1",
            "not t1, t1",
            "and t0, t0, t1",
            "csrw mstatus, t0",

            "la   sp, __stack_top",

            // Allow the supervisor to read/write/execute anywhere between 0-0x2fffff..
            "li t0, 0x2fffffffffffffff",
            "csrw pmpaddr0, t0",
            "csrw pmpcfg0, {pmp_cfg}",

            "mret",

            entry = in(reg) entry,
            mpp_s = const MSTATUS_MPP_S,
            pmp_cfg = const PMP_0_CFG,
            options(noreturn)
        )
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn start() -> ! {
    b"hello from the supervisor\n"
        .into_iter()
        .for_each(|b| unsafe { core::ptr::write_volatile(UART_ADDR, *b) });

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn pnic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
