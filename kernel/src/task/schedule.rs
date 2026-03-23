use crate::{
    KERNEL, PROC_TABLE,
    arch::{self, TrapFrame},
    helper::{u64_to_str, u64_to_str_hex},
    kdebug, ktrace,
};

pub fn schedule(reset_timer: bool) {
    let mut buf = [0; 20];

    let current_proc_id = unsafe { KERNEL.current_running_proc } as u64;

    let current_process = unsafe { PROC_TABLE[current_proc_id as usize].assume_init_mut() };

    match find_next_available_proc_id(current_proc_id as usize) {
        Some(next_proc_id) => {
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
                    (*tf).sepc = crate::task::PROCESS_TEXT_ADDRESS.raw() as usize;
                    (*tf).sp = crate::task::PROCESS_STACK_ADDRESS.raw() as usize - 4;
                    (*tf).sstatus = riscv::registers::Sstatus::read()
                        .enable_user_mode()
                        .enable_supervisor_interrupts()
                        .enable_user_page_access()
                        .raw() as usize;
                }
                next_process.context.sp = tf as u64;
                next_process.context.ra = arch::trap_resume as *const () as u64;
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

            next_process.state = crate::ProcessState::Running;

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
                arch::swtch(
                    (&mut current_process.context) as *mut arch::Context,
                    (&next_process.context) as *const arch::Context,
                );
            }
        }
        None => unsafe {
            let idle_process = PROC_TABLE[0].assume_init_mut();
            idle_process.ticks_at_started_running = riscv::registers::Time::read().raw();
            ktrace("scheduling bro\n");
            if current_proc_id == 0 {
                return;
            }
            riscv::registers::Sscratch::new(0).write();
            KERNEL.current_running_proc = 0;
            arch::swtch(
                (&mut current_process.context) as *mut arch::Context,
                (&PROC_TABLE[0].assume_init_ref().context) as *const arch::Context,
            );
        },
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
            crate::task::ProcessState::Sleeping => {
                if time > proc.wake_up_at {
                    proc.wake_up_at = 0;
                    return Some(current_proc_id);
                }
            }
            crate::task::ProcessState::Running => {}
            crate::task::ProcessState::Ready => {
                crate::ktrace("going to run: \n");
                let mut buf = [0; 20];
                crate::ktrace(u64_to_str(current_proc_id as u64, &mut buf));
                return Some(current_proc_id);
            }
            crate::task::ProcessState::Blocked => {}
        }
    }

    None
}
