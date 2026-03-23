use core::arch::global_asm;

use crate::{
    KERNEL, PROC_TABLE, SYSCALL_READ, SYSCALL_SHUTDOWN, SYSCALL_SLEEP_MS, SYSCALL_WRITE,
    arch::{Context, mmu::VirtualAddress},
    console, ktrace, plic, task,
};

unsafe extern "C" {
    #[allow(unused)]
    pub fn swtch(from: *mut Context, to: *const Context);

    pub fn trap_resume();
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

trap_resume:
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
);

// TODO: should we represent registers as signed instead?
#[repr(C)]
#[derive(Clone, Default)]
pub struct TrapFrame {
    pub ra: usize,
    pub sp: usize,
    pub gp: usize,
    pub tp: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub s0: usize,
    pub s1: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,

    pub sepc: usize,
    pub scause: usize,
    pub sstatus: usize,
}

impl TrapFrame {
    /// Initializes a trap frame for a new task
    pub fn initialize(instruction_ptr: VirtualAddress, stack_ptr: VirtualAddress) -> Self {
        Self {
            sepc: instruction_ptr.raw() as usize,
            sp: stack_ptr.raw() as usize,
            sstatus: riscv::registers::Sstatus::empty()
                .enable_user_mode()
                .enable_supervisor_interrupts()
                .enable_user_page_access()
                .raw() as usize,
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
                                PROC_TABLE[*idx].assume_init_mut().state =
                                    crate::task::ProcessState::Ready;
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
            ktrace("timer interrupt\n");
            let current_process =
                unsafe { PROC_TABLE[KERNEL.current_running_proc].assume_init_mut() };

            let nanos = |ticks: u64| ticks * 1_000_000_000 / 10_000_000;

            let current_ticks = riscv::registers::Time::read().raw();

            // 32ms
            if nanos(current_ticks) - nanos(current_process.ticks_at_started_running)
                > 4_000_000 * 8
            {
                ktrace("time is up, we are scheduling\n");
                unsafe {
                    PROC_TABLE[KERNEL.current_running_proc]
                        .assume_init_mut()
                        .state = crate::task::ProcessState::Ready;
                }
                task::schedule(true);
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

                    let ms_to_ticks = |ms: u64| ms * 10_000_000 / 1_000;

                    current_process.state = crate::task::ProcessState::Sleeping;

                    current_process.wake_up_at =
                        riscv::registers::Time::read().raw() + ms_to_ticks(ms);

                    task::schedule(false);
                }
                SYSCALL_SHUTDOWN => {
                    crate::halt();
                }
                _ => unreachable!(),
            }
        }
        trap => {
            crate::kdebug("unhandled trap\n\t");
            crate::kdebug(crate::u64_to_str(trap as u64, &mut [0; 20]));
            crate::kdebug("stval: \n\t");
            crate::kdebug(crate::u64_to_str_hex(
                riscv::registers::Stval::read().raw(),
                &mut [0; 32],
            ));
            crate::kdebug("sepc: \n\t");
            crate::kdebug(crate::u64_to_str_hex(
                riscv::registers::Sepc::read().raw(),
                &mut [0; 32],
            ));
            unreachable!()
        }
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
                            .state = crate::ProcessState::Blocked;
                    }
                    unsafe {
                        KERNEL.uart_wait_queue[KERNEL.uart_wait_queue_len] =
                            KERNEL.current_running_proc;
                        KERNEL.uart_wait_queue_len += 1;
                    }

                    task::schedule(false);
                }
            }
        }
    }

    i
}
