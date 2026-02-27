#![no_std]
#![no_main]

use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};

global_asm!(include_str!("start.s"));

const UART_ADDR: *mut u8 = 0x10000000 as *mut u8;

const MSTATUS_MPP_SHIFT: usize = 11;
const MSTATUS_MPP_MASK: usize = 0b11 << MSTATUS_MPP_SHIFT;
const MSTATUS_MPP_S: usize = 0b01 << MSTATUS_MPP_SHIFT;

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    // b"hello world from kernel"
    //     .into_iter()
    //     .for_each(|b| unsafe { core::ptr::write_volatile(UART_ADDR, *b) });

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

            "csrr t0, mstatus",
            "li   t1, {mpp_mask}",
            "not  t1, t1",
            "and  t0, t0, t1",
            "li   t1, {mpp_s}",
            "or   t0, t0, t1",
            "csrw mstatus, t0",

            "la   sp, __stack_top",

            "mret",

            entry = in(reg) entry,
            mpp_mask = const MSTATUS_MPP_MASK,
            mpp_s = const MSTATUS_MPP_S,
            options(noreturn)
        )
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn start() -> ! {
    b"hello from the supervisor"
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
