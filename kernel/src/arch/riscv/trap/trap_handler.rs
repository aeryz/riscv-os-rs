use crate::{
    KERNEL, PROC_TABLE, SYSCALL_READ, SYSCALL_SHUTDOWN, SYSCALL_SLEEP_MS, SYSCALL_WRITE,
    arch::TrapFrame, console, ktrace, plic, task,
};

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
