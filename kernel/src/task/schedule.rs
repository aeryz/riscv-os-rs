use crate::{
    Arch,
    arch::{Architecture, ContextOf, MemoryModel},
    helper::u64_to_str,
    kdebug,
    task::{self, Process},
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
            let next_process = task::get_process_at_mut(next_proc_id);

            Arch::set_root_page_table(next_process.root_table);

            next_process.ticks_at_started_running = Arch::read_current_time();

            if reset_timer {
                // 4ms
                Arch::set_timer(4 * 10_000_000 / 1_000 + next_process.ticks_at_started_running);
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

            Arch::switch(
                (&mut current_process.context) as *mut ContextOf<Arch>,
                (&next_process.context) as *const ContextOf<Arch>,
            );
        }
        None => {
            let idle_process = task::get_process_at_mut(0);
            idle_process.ticks_at_started_running = Arch::read_current_time();
            if ctx.current_running_proc_idx == 0 {
                return;
            }
            ctx.current_running_proc_idx = 0;
            Arch::set_kernel_sp(0);
            Arch::switch(
                (&mut current_process.context) as *mut ContextOf<Arch>,
                &idle_process.context as *const ContextOf<Arch>,
            );
        }
    }
}

pub fn get_currently_running_process() -> &'static Process {
    let scheduler = unsafe { &SCHEDULER_CTX };
    task::get_process_at(scheduler.current_running_proc_idx)
}

pub fn get_currently_running_process_mut() -> &'static mut Process {
    let scheduler = unsafe { &SCHEDULER_CTX };
    task::get_process_at_mut(scheduler.current_running_proc_idx)
}

fn find_next_available_proc_id(ctx: &Scheduler) -> Option<usize> {
    let time = Arch::read_current_time();

    for proc in task::iterate_process_table_mut(ctx.current_running_proc_idx) {
        if proc.pid == 0 {
            continue;
        }
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
