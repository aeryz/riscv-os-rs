use crate::{
    KERNEL,
    arch::riscv::trap::trap_frame::{TrapCause, TrapFrame},
    ktrace, plic, syscall, task,
};

#[unsafe(no_mangle)]
extern "C" fn trap_handler(trap_frame: &mut TrapFrame) {
    // https://docs.riscv.org/reference/isa/priv/supervisor.html#scause
    match trap_frame.get_cause() {
        // TODO(aeryz): right now, we don't have ISA-independent drivers. Keeping this as is
        // right now but this is no good.
        TrapCause::ExternalIrq => {
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
                                task::get_process_at_mut(*idx).state =
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
        TrapCause::TimerInterrupt => {
            task::handle_timer_interrupt();
        }
        TrapCause::Syscall => {
            // This is a syscall, so we move the return program counter to just after the `ecall`
            trap_frame.sepc += 4;
            syscall::dispatch_syscall(trap_frame);
        }
        TrapCause::Unknown(trap) => {
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
            panic!();
        }
    }
}
