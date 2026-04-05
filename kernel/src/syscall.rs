use crate::{
    Arch, KERNEL,
    arch::{Architecture, TrapFrame, TrapFrameOf},
    console, ktrace, task,
};

pub const SYSCALL_WRITE: usize = 1;
pub const SYSCALL_READ: usize = 2;
pub const SYSCALL_SLEEP_MS: usize = 3;
pub const SYSCALL_SHUTDOWN: usize = 4;

// TODO(aeryz): We don't want to implement the syscalls here. But they should directly be implemented
// in their respective subsystem.

pub fn dispatch_syscall(tf: &mut TrapFrameOf<Arch>) {
    let syscall_number = tf.get_syscall();
    match syscall_number {
        SYSCALL_WRITE => {
            let _fd = tf.get_arg::<0>();
            let buf = tf.get_arg::<1>() as *const u8;
            let count = tf.get_arg::<2>();

            let utf8_str = unsafe { core::slice::from_raw_parts(buf, count) };

            console::print(utf8_str);

            tf.set_syscall_return_value(count);
        }
        SYSCALL_READ => {
            let _fd = tf.get_arg::<0>();
            let buf = tf.get_arg::<1>() as *mut u8;
            let count = tf.get_arg::<2>();

            let buf = unsafe { core::slice::from_raw_parts_mut(buf, count) };

            let n_read = syscall_read(buf);
            tf.set_syscall_return_value(n_read);
        }
        SYSCALL_SLEEP_MS => {
            let ms = tf.get_arg::<0>();

            let current_process = task::get_currently_running_process_mut();

            let ms_to_ticks = |ms| ms * 10_000_000 / 1_000;

            current_process.state = crate::task::ProcessState::Sleeping;

            current_process.wake_up_at = Arch::read_current_time() + ms_to_ticks(ms);

            task::schedule(false);
        }
        SYSCALL_SHUTDOWN => {
            crate::halt();
        }
        _ => unreachable!(),
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
                    let current_process = task::get_currently_running_process_mut();
                    current_process.state = task::ProcessState::Blocked;
                    unsafe {
                        KERNEL.uart_wait_queue[KERNEL.uart_wait_queue_len] = current_process.pid;
                        KERNEL.uart_wait_queue_len += 1;
                    }

                    task::schedule(false);
                }
            }
        }
    }

    i
}
