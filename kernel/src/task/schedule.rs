use crate::{
    arch::{self},
    helper::u64_to_str,
    kdebug, ktrace, task,
};

static mut SCHEDULER_CTX: Scheduler = Scheduler {
    current_running_proc_idx: 1,
};

struct Scheduler {
    current_running_proc_idx: usize,
}

pub fn schedule(reset_timer: bool) {
    let mut buf = [0; 20];

    let ctx = unsafe { &mut SCHEDULER_CTX };

    let current_process = task::get_process_at_mut(ctx.current_running_proc_idx);

    match find_next_available_proc_id(ctx) {
        Some(next_proc_id) => {
            ctx.current_running_proc_idx = next_proc_id;
            let next_process = task::get_process_at_mut(next_proc_id);

            arch::mmu::set_root_page_table(next_process.root_table);

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
            kdebug(u64_to_str(ctx.current_running_proc_idx as u64, &mut buf));

            kdebug("switching to: \n\t");
            kdebug(u64_to_str(next_proc_id as u64, &mut buf));

            if ctx.current_running_proc_idx == next_proc_id {
                return;
            }

            ctx.current_running_proc_idx = next_proc_id;

            unsafe {
                arch::swtch(
                    (&mut current_process.context) as *mut arch::Context,
                    (&next_process.context) as *const arch::Context,
                );
            }
        }
        None => unsafe {
            let idle_process = task::get_process_at_mut(0);
            idle_process.ticks_at_started_running = riscv::registers::Time::read().raw();
            if ctx.current_running_proc_idx == 0 {
                return;
            }
            riscv::registers::Sscratch::new(0).write();
            ctx.current_running_proc_idx = 0;
            arch::swtch(
                (&mut current_process.context) as *mut arch::Context,
                &idle_process.context as *const arch::Context,
            );
        },
    }
}

fn find_next_available_proc_id(ctx: &Scheduler) -> Option<usize> {
    let time = riscv::registers::Time::read().raw();

    // We bypass the idle task by doing `KERNEL.n_procs - 1` iterations
    for proc in task::iterate_process_table_mut(ctx.current_running_proc_idx) {
        match proc.state {
            crate::task::ProcessState::Sleeping => {
                if time > proc.wake_up_at {
                    proc.wake_up_at = 0;
                    return Some(proc.pid);
                }
            }
            crate::task::ProcessState::Running => {}
            crate::task::ProcessState::Ready => {
                crate::ktrace("going to run: \n");
                let mut buf = [0; 20];
                crate::ktrace(u64_to_str(proc.pid as u64, &mut buf));
                return Some(proc.pid);
            }
            crate::task::ProcessState::Blocked => {}
        }
    }

    None
}
