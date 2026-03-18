use core::arch::global_asm;

use crate::{
    KERNEL, PROC_TABLE, SYSCALL_READ, SYSCALL_WRITE, console,
    context::Context,
    helper::{u64_to_str, u64_to_str_hex},
    kdebug, ktrace, plic,
};

unsafe extern "C" {
    #[allow(unused)]
    fn swtch(from: *mut Context, to: *const Context);

    fn trap_resume();
}

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
    .globl trap_resume
    .align 2
trap_entry:

    // Swap the kernel and user stacks
    csrrw sp, sscratch, sp

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
   
    // Move the trap frame (sitting at sp) as the first param
    mv a0, sp
    call trap_handler

trap_resume:
    ld t0, 31*8(sp)
    csrw sepc, t0

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
    sret
"#,
    TRAPFRAME_SIZE = const size_of::<TrapFrame>(),
    READ_SP = const (size_of::<TrapFrame>() - 8),
);

// TODO: should we represent registers as signed instead?
#[repr(C)]
#[derive(Clone, Default)]
pub struct TrapFrame {
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

    sepc: usize,
    scause: usize,
}

impl TrapFrame {
    pub fn with_sepc(sepc: u64) -> Self {
        Self {
            sepc: sepc as usize,
            ..Default::default()
        }
    }
}

#[unsafe(no_mangle)]
extern "C" fn trap_handler(trap_frame: &mut TrapFrame) {
    trap_frame.a0 = 0xffffffffffffffff; // -1
    // https://docs.riscv.org/reference/isa/priv/supervisor.html#scause
    match trap_frame.scause {
        // I = 1, C = 9 = supervisor external interrupt
        0x8000000000000009 => {
            // TODO: only support the hart = 0
            let interrupt_id = plic::plic_claim(0);
            match interrupt_id {
                crate::plic::UART0_IRQ => {
                    kdebug("this is a uart interrupt: ");

                    while let Some(_val) = unsafe { crate::UART.read_char_into_buffer() } {
                        // TODO: can debug here
                    }
                    plic::plic_complete(0, crate::plic::UART0_IRQ);
                }
                _ => {
                    kdebug("i dont know this interrupt sorry");
                }
            }
        }
        // I = 0, C = 8 = environment call from U-Mode
        8 => {
            // This is a syscall, so we move the return program counter to just after the `ecall`
            trap_frame.sepc += 4;
            let syscall_number = trap_frame.a7;
            match syscall_number {
                SYSCALL_WRITE => {
                    let _fd = trap_frame.a0;
                    let buf = trap_frame.a1 as *const u8;
                    let count = trap_frame.a2;

                    let utf8_str = unsafe { core::slice::from_raw_parts(buf, count) };

                    console::print(utf8_str);

                    trap_frame.a0 = count;
                }
                SYSCALL_READ => {
                    let _fd = trap_frame.a0;
                    let buf = trap_frame.a1 as *mut u8;
                    let count = trap_frame.a2;

                    let buf = unsafe { core::slice::from_raw_parts_mut(buf, count) };

                    let n_read = syscall_read(buf);
                    trap_frame.a0 = n_read;
                }
                _ => unreachable!(),
            }

            schedule()
        }
        _ => {
            unreachable!()
        }
    }
}

fn schedule() {
    let mut buf = [0; 20];

    let mut current_proc_id = unsafe { KERNEL.current_running_proc } as u64;

    let current_process = unsafe { PROC_TABLE[current_proc_id as usize].assume_init_mut() };

    ktrace("current proc: \n\t");
    ktrace(u64_to_str(current_proc_id, &mut buf));

    if current_proc_id + 1 >= unsafe { KERNEL.n_procs } as u64 {
        current_proc_id = 0;
        unsafe {
            KERNEL.current_running_proc = 0;
        }
    } else {
        current_proc_id += 1;
        unsafe {
            KERNEL.current_running_proc += 1;
        }
    }

    let process = unsafe { PROC_TABLE[current_proc_id as usize].assume_init_mut() };

    ktrace("switching to: \n\t");
    ktrace(u64_to_str(current_proc_id, &mut buf));

    riscv::registers::Satp::empty()
        .set_mode(riscv::registers::SatpMode::Sv39)
        .set_ppn(process.root_table_pa)
        .write();

    if process.trap_frame.is_null() {
        let tf = (process.kernel_sp - size_of::<TrapFrame>() as u64) as *mut TrapFrame;
        ktrace("trap frame is null, so setting it to: ");
        ktrace(u64_to_str_hex(tf as u64, &mut buf));
        process.trap_frame = tf;

        unsafe {
            (*tf).sepc = crate::process::PROC_TEXT_VA as usize;
            (*tf).sp = crate::process::PROC_STACK_VA as usize - 4;
        }
        process.context.sp = tf as u64;
        process.context.ra = trap_resume as *const () as u64;
    } else {
        ktrace("trap frame is not null\n");
    }

    // TODO: this is UB if the `current_process` == `process`
    unsafe {
        swtch(
            (&mut current_process.context) as *mut Context,
            (&process.context) as *const Context,
        );
    }
}

pub fn syscall_read(buf: &mut [u8]) -> usize {
    let mut i = 0;
    while i < buf.len() {
        loop {
            match unsafe { crate::UART.try_get_char() } {
                Some(c) => {
                    kdebug("read something, not scheduling");
                    if c == b'\n' || c == b'\r' {
                        return i;
                    }
                    buf[i] = c;
                    i += 1;
                    break;
                }
                None => {
                    kdebug("couldn't read anything, scheduling");
                    schedule()
                }
            }
        }
    }

    i
}
