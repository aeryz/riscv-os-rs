use core::arch::global_asm;

use crate::{
    KERNEL, PROC_TABLE, SYSCALL_READ, SYSCALL_SLEEP_MS, SYSCALL_WRITE, console,
    context::Context,
    helper::{u64_to_str, u64_to_str_hex},
    kdebug, ktrace, plic, process,
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
    unsafe {
        PROC_TABLE[KERNEL.current_running_proc]
            .assume_init_mut()
            .trap_frame = trap_frame as *mut TrapFrame;
    }
    trap_frame.a0 = 0xffffffffffffffff; // -1
    // https://docs.riscv.org/reference/isa/priv/supervisor.html#scause
    match trap_frame.scause {
        // I = 1, C = 9 = supervisor external interrupt
        0x8000000000000009 => {
            // TODO: only support the hart = 0
            let interrupt_id = plic::plic_claim(0);
            match interrupt_id {
                crate::plic::UART0_IRQ => {
                    ktrace("this is a uart interrupt: \n");

                    let mut read_anything = false;
                    while let Some(_val) = unsafe { crate::UART.read_char_into_buffer() } {
                        read_anything = true;
                        // TODO: can debug here
                    }

                    if read_anything {
                        unsafe {
                            // Whenever a read happens, iterate through the uart queue and set all the waiting processes to
                            // ready.
                            for idx in KERNEL.uart_wait_queue.iter_mut().take_while(|i| **i != 0) {
                                PROC_TABLE[*idx].assume_init_mut().state = process::State::Ready;
                                *idx = 0;
                            }
                            KERNEL.uart_wait_queue_len = 0;
                        }
                    }

                    plic::plic_complete(0, crate::plic::UART0_IRQ);
                }
                _ => {
                    ktrace("i dont know this interrupt sorry\n");
                }
            }
        }
        // I = 1, C = 5 = timer tick
        0x8000000000000005 => {
            ktrace("timer interrupt");
            let current_process =
                unsafe { PROC_TABLE[KERNEL.current_running_proc].assume_init_mut() };

            let nanos = |ticks: u64| ticks * 1_000_000_000 / 10_000_000;

            let current_ticks = riscv::registers::Time::read().raw();

            // 32ms
            if nanos(current_ticks) - nanos(current_process.ticks_at_started_running)
                > 4_000_000 * 8
            {
                unsafe {
                    PROC_TABLE[KERNEL.current_running_proc]
                        .assume_init_mut()
                        .state = process::State::Ready;
                }
                schedule(true);
            } else {
                // 4ms
                riscv::registers::Stimecmp::new(4 * 10_000_000 / 1_000 + current_ticks).write();
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
                SYSCALL_SLEEP_MS => {
                    let ms = trap_frame.a0 as u64;

                    let current_process =
                        unsafe { PROC_TABLE[KERNEL.current_running_proc].assume_init_mut() };

                    let ms_to_ticks = |ticks: u64| ticks * 10_000_000 / 1_000;

                    current_process.state = process::State::Sleeping;
                    current_process.wake_up_at =
                        riscv::registers::Time::read().raw() + ms_to_ticks(ms);

                    schedule(false);
                }
                _ => unreachable!(),
            }
        }
        _ => {
            unreachable!()
        }
    }
}

fn find_next_available_proc_id(mut current_proc_id: usize) -> Option<usize> {
    let time = riscv::registers::Time::read().raw();

    // We bypass the idle task by doing `KERNEL.n_procs - 1` iterations
    for _ in unsafe { 0..(KERNEL.n_procs - 1) } {
        if current_proc_id + 1 >= unsafe { KERNEL.n_procs } {
            // We bypass the idle task here
            current_proc_id = 1;
        } else {
            current_proc_id += 1;
        }
        let proc = unsafe { PROC_TABLE[current_proc_id as usize].assume_init_mut() };

        match proc.state {
            process::State::Sleeping => {
                if time > proc.wake_up_at {
                    proc.wake_up_at = 0;
                    return Some(current_proc_id);
                }
            }
            process::State::Running => {}
            process::State::Ready => {
                crate::ktrace("going to run: \n");
                let mut buf = [0; 20];
                crate::ktrace(u64_to_str(current_proc_id as u64, &mut buf));
                return Some(current_proc_id);
            }
            process::State::Blocked => {}
        }
    }

    None
}

fn schedule(reset_timer: bool) {
    let mut buf = [0; 20];

    let current_proc_id = unsafe { KERNEL.current_running_proc } as u64;

    let current_process = unsafe { PROC_TABLE[current_proc_id as usize].assume_init_mut() };

    let next_proc_id = find_next_available_proc_id(current_proc_id as usize).unwrap_or(0);

    let next_process = unsafe { PROC_TABLE[next_proc_id].assume_init_mut() };

    riscv::registers::Satp::empty()
        .set_mode(riscv::registers::SatpMode::Sv39)
        .set_ppn(next_process.root_table_pa)
        .write();

    if next_process.trap_frame.is_null() {
        let tf = (next_process.kernel_sp - size_of::<TrapFrame>() as u64) as *mut TrapFrame;
        ktrace("trap frame is null, so setting it to: ");
        ktrace(u64_to_str_hex(tf as u64, &mut buf));
        next_process.trap_frame = tf;

        unsafe {
            (*tf).sepc = crate::process::PROC_TEXT_VA as usize;
            (*tf).sp = crate::process::PROC_STACK_VA as usize - 4;
        }
        next_process.context.sp = tf as u64;
        next_process.context.ra = trap_resume as *const () as u64;
    } else {
        ktrace("trap frame is not null\n");
    }

    next_process.ticks_at_started_running = riscv::registers::Time::read().raw();

    if reset_timer {
        // 4ms
        riscv::registers::Stimecmp::new(
            4 * 10_000_000 / 1_000 + next_process.ticks_at_started_running,
        )
        .write();
    }

    next_process.state = process::State::Running;

    kdebug("current proc: \n\t");
    kdebug(u64_to_str(current_proc_id, &mut buf));

    kdebug("switching to: \n\t");
    kdebug(u64_to_str(next_proc_id as u64, &mut buf));

    if current_proc_id == next_proc_id as u64 {
        return;
    }

    unsafe {
        KERNEL.current_running_proc = next_proc_id;
    }

    unsafe {
        swtch(
            (&mut current_process.context) as *mut Context,
            (&next_process.context) as *const Context,
        );
    }
}

pub fn syscall_read(buf: &mut [u8]) -> usize {
    let mut i = 0;
    while i < buf.len() {
        loop {
            match unsafe { crate::UART.try_get_char() } {
                Some(c) => {
                    ktrace("read something, not scheduling\n");
                    if c == b'\n' || c == b'\r' {
                        return i;
                    }
                    buf[i] = c;
                    i += 1;
                    break;
                }
                None => {
                    ktrace("couldn't read anything, scheduling\n");
                    unsafe {
                        PROC_TABLE[KERNEL.current_running_proc]
                            .assume_init_mut()
                            .state = process::State::Blocked;
                    }
                    unsafe {
                        KERNEL.uart_wait_queue[KERNEL.uart_wait_queue_len] =
                            KERNEL.current_running_proc;
                        KERNEL.uart_wait_queue_len += 1;
                    }

                    schedule(false);
                }
            }
        }
    }

    i
}
