#![no_std]
#![no_main]

use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};

global_asm!(include_str!("start.s"));

const UART_ADDR: *mut u8 = 0x10000000 as *mut u8;

const XSTATUS_XPP_SHIFT: usize = 11;
const XSTATUS_XPP_S: usize = 0b01 << XSTATUS_XPP_SHIFT;
const XSTATUS_MPP_X: usize = 0b11 << XSTATUS_XPP_SHIFT;
const XSTATUS_SIE: usize = 0b1 << 1;
const PMP_0_CFG: usize = 0b00001111;

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    // b"hello world from kernel\n"
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
            "li t1, {xpp_s}",
            // unset 12th bit for setting the MPP to 01(S mode)
            "or t0, t0, t1",
            "slli t1, t1, 1",
            "not t1, t1",
            "and t0, t0, t1",
            "csrw mstatus, t0",

            // TODO: delegating everything to supervisor right now for ease of use.
            // Need to investigate further to see if we want to handle some traps
            // in the M-level.
            // Delegate all interrupts and traps to the supervisor
            "li t0, -1",
            "csrw medeleg, t0",
            "csrw mideleg, t0",

            "la   sp, __stack_top",

            // Allow the supervisor to read/write/execute anywhere between 0-0x2fffff..
            "li t0, 0x2fffffffffffffff",
            "csrw pmpaddr0, t0",
            "csrw pmpcfg0, {pmp_cfg}",

            "mret",

            entry = in(reg) entry,
            xpp_s = const XSTATUS_XPP_S,
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

    enter_usermode(
        userspace_init as *const () as usize,
        trap_handler as *const () as usize,
    );

    loop {
        core::hint::spin_loop();
    }
}

#[inline(always)]
pub fn enter_usermode(entry: usize, trap_handler: usize) {
    unsafe {
        asm!(
            "csrw sepc, {entry}",

            "csrr t0, sstatus",
            // set spp to usermode (00)
            "li t1, {xpp_m}",
            "not t1, t1",
            "and t0, t0, t1",

            // enable trap handler
            "ori t0, t0, {sie}",

            "csrw sstatus, t0",

            // setup the trap handler base address
            "csrw stvec, {trap_handler}",

            // TODO: enable scounteren

            "sret",

            entry = in(reg) entry,
            trap_handler = in(reg) trap_handler,
            xpp_m = const XSTATUS_MPP_X,
            sie = const XSTATUS_SIE,

            options(noreturn)
        )
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn userspace_init() -> ! {
    b"hello from the userspace\n"
        .into_iter()
        .for_each(|b| unsafe { core::ptr::write_volatile(UART_ADDR, *b) });

    unsafe {
        asm!("ecall",
            lateout("a0") _,
            lateout("a1") _,
            lateout("a2") _,
            lateout("a3") _,
            lateout("a4") _,
            lateout("a5") _,
            lateout("t0") _,
        )
    }

    b"hello from the userspace after the ecall\n"
        .into_iter()
        .for_each(|b| unsafe { core::ptr::write_volatile(UART_ADDR, *b) });

    loop {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn trap_handler() -> ! {
    b"this is a fuckin trap\n"
        .into_iter()
        .for_each(|b| unsafe { core::ptr::write_volatile(UART_ADDR, *b) });

    unsafe {
        asm!(
            // increment sepc to return to the next instr after `ecall`
            "csrr t0, sepc",
            "addi t0, t0, 4", // ecall is 4 bytes
            "csrw sepc, t0",
            "sret",
        )
    }

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
