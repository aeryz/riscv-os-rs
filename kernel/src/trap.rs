use core::arch::global_asm;

use crate::{SYSCALL_WRITE, UART_ADDR};

// The trampoline to save the trap frame and jump to the high level trap handler.
// This is required because:
// 1. `stvec` (trap handler address) needs to be 4-byte aligned.
// 2. The registers need to be saved before calling the high level trap handler so that
// we make sure the high level function can access to the unmodified registers and we preserve
// the the registers.
global_asm!(
    r#"
    .section .text.trap
    .globl trap_entry
    .align 2
trap_entry:

    // Swap the kernel and user stacks
    csrrw sp, sscratch, sp

    // Allocate the stack pointer to fit the trapframe
    // TODO: maybe use sizeof(TrapFrame) here?
    addi sp, sp, -{TRAPFRAME_SIZE}

    // Now we can start saving the registers into the trap frame.
    // Otherwise, there is no guarantee that our registers will not be
    // altered with. (had a painful experience with this)
    sd ra,  0*8(sp)
    sd sp,  1*8(sp)
    sd gp,  2*8(sp)
    sd tp,  3*8(sp)
    sd t0,  4*8(sp)
    sd t1,  5*8(sp)
    sd t2,  6*8(sp)
    sd s0,  7*8(sp)
    sd s1,  8*8(sp)
    sd a0,  9*8(sp)
    sd a1,  10*8(sp)
    sd a2,  11*8(sp)
    sd a3,  12*8(sp)
    sd a4,  13*8(sp)
    sd a5,  14*8(sp)
    sd a6,  15*8(sp)
    sd a7,  16*8(sp)
    sd s2,  17*8(sp)
    sd s3,  18*8(sp)
    sd s4,  19*8(sp)
    sd s5,  20*8(sp)
    sd s6,  21*8(sp)
    sd s7,  22*8(sp)
    sd s8,  23*8(sp)
    sd s9,  24*8(sp)
    sd s10, 25*8(sp)
    sd s11, 26*8(sp)
    sd t3,  27*8(sp)
    sd t4,  28*8(sp)
    sd t5,  29*8(sp)
    sd t6,  30*8(sp)

    csrr t0, scause
    sd t0, 31*8(sp)
    
    // Move the trap frame (sitting at sp) as the first param
    mv a0, sp
    call trap_handler

    ld ra,  0*8(sp)
    ld sp,  1*8(sp)
    ld gp,  2*8(sp)
    ld tp,  3*8(sp)
    ld t0,  4*8(sp)
    ld t1,  5*8(sp)
    ld t2,  6*8(sp)
    ld s0,  7*8(sp)
    ld s1,  8*8(sp)
    ld a0,  9*8(sp)
    ld a1,  10*8(sp)
    ld a2,  11*8(sp)
    ld a3,  12*8(sp)
    ld a4,  13*8(sp)
    ld a5,  14*8(sp)
    ld a6,  15*8(sp)
    ld a7,  16*8(sp)
    ld s2,  17*8(sp)
    ld s3,  18*8(sp)
    ld s4,  19*8(sp)
    ld s5,  20*8(sp)
    ld s6,  21*8(sp)
    ld s7,  22*8(sp)
    ld s8,  23*8(sp)
    ld s9,  24*8(sp)
    ld s10, 25*8(sp)
    ld s11, 26*8(sp)
    ld t3,  27*8(sp)
    ld t4,  28*8(sp)
    ld t5,  29*8(sp)
    ld t6,  30*8(sp)

    // Restore the stack pointer
    // TODO: maybe use sizeof(TrapFrame) here?
    addi sp, sp, {TRAPFRAME_SIZE}

    // Increment `sepc` to return to the next instr after `ecall`
    csrr t0, sepc
    // `ecall` is 4 bytes
    addi t0, t0, 4 
    csrw sepc, t0
    // Swap the user and kernel stacks back
    csrrw sp, sscratch, sp
    sret
"#,
    TRAPFRAME_SIZE = const size_of::<TrapFrame>(),
);

// TODO: should we represent registers as signed instead?
#[repr(C)]
struct TrapFrame {
    ra: usize,
    sp: usize,
    gp: usize,
    tp: usize,
    t0: usize,
    t1: usize,
    t2: usize,
    s0: usize,
    s1: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
    a7: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    s9: usize,
    s10: usize,
    s11: usize,
    t3: usize,
    t4: usize,
    t5: usize,
    t6: usize,

    scause: usize,
}

#[unsafe(no_mangle)]
extern "C" fn trap_handler(trap_frame: &mut TrapFrame) {
    trap_frame.a0 = 0xffffffffffffffff; // -1
    match trap_frame.scause {
        // 8 = environment call from U-Mode
        // https://docs.riscv.org/reference/isa/priv/supervisor.html#scause
        8 => {
            let syscall_number = trap_frame.a7;
            if syscall_number == SYSCALL_WRITE {
                let _fd = trap_frame.a0;
                let buf = trap_frame.a1 as *const u8;
                let count = trap_frame.a2;

                let utf8_str = unsafe { core::slice::from_raw_parts(buf, count) };

                utf8_str.into_iter().for_each(|b| unsafe {
                    core::ptr::write_volatile(UART_ADDR, *b);
                });

                trap_frame.a0 = count;
            }
        }
        _ => {}
    }
}
