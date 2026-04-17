use crate::arch::trap::trap_frame::TrapFrame;

#[unsafe(naked)]
pub extern "C" fn trap_entry() -> ! {
    core::arch::naked_asm!(
        r#"
    // Swap the kernel and user stacks
    csrrw sp, sscratch, sp

    // if sp = 0, then this is a kernel process and we should load sp from scratch
    bnez sp, save_trapframe
    csrr sp, sscratch

save_trapframe:
    // Allocate the stack pointer to fit the trapframe
    addi sp, sp, -{TRAPFRAME_SIZE}

    // Now we can start saving the registers into the trap frame.
    // Otherwise, there is no guarantee that our registers will not be
    // altered with. (had a painful experience with this)
    sd ra,  0*8(sp)

    // read the user sp
    csrr ra, sscratch
    sd ra,  1*8(sp)
    // then restore the ra
    ld ra,  0*8(sp)
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

    csrr t0, sepc
    sd t0, 31*8(sp)

    csrr t0, scause
    sd t0, 32*8(sp)

    csrr t0, sstatus
    sd t0, 33*8(sp)
   
    // Move the trap frame (sitting at sp) as the first param
    mv a0, sp
    call trap_handler
        "#,
    TRAPFRAME_SIZE = const size_of::<TrapFrame>(),
    );
}

#[unsafe(naked)]
pub extern "C" fn trap_resume() {
    core::arch::naked_asm!(
        r#"
    ld t0, 31*8(sp)
    csrw sepc, t0

    ld t0, 33*8(sp)
    csrw sstatus, t0

    ld ra,  0*8(sp)
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
    addi sp, sp, {TRAPFRAME_SIZE}
    csrw sscratch, sp

    ld sp, -{READ_SP}(sp)

    // if sp != 0, then this is a userspace program and we should return to it, otherwise we must
    // load the sp back from sscratch and do regular ret
    bnez sp, ret_userspace

    csrrw sp, sscratch, sp
    ret
ret_userspace:
    sret
        "#,
    TRAPFRAME_SIZE = const size_of::<TrapFrame>(),
    READ_SP = const (size_of::<TrapFrame>() - 8),
    )
}
