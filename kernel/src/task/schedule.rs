use crate::{
    Arch,
    arch::{Architecture, ContextOf, MemoryModel},
    helper::u64_to_str,
    kdebug, ktrace,
    task::{self, Process, TASK_PID_IDLE, TASK_PID_REAPER},
};

/// Defines how long can a process run on CPU before being scheduled.
pub const PER_PROCESS_TIME_SLICE_NANOS: usize = 4_000_000 * 8 /* 32 ms */;

static mut SCHEDULER_CTX: Scheduler = Scheduler {
    current_running_proc_idx: 0,
};

struct Scheduler {
    current_running_proc_idx: usize,
}

pub fn init_scheduler(initial_proc_pid: usize) {
    unsafe { SCHEDULER_CTX.current_running_proc_idx = initial_proc_pid }
}

/// Handles a timer interrupt
///
/// Determines whether a timer interrupt should result in scheduling or not.
pub fn handle_timer_interrupt() {
    let current_process_pid = get_currently_running_process().pid;

    let current_ticks = Arch::read_current_time();

    // If we are running in the idle task, it means we can have sleeping tasks that are
    // ready to be woken up. So if we are in the idle task and we find any task like that,
    // we do an early switch to the target.
    if current_process_pid == TASK_PID_IDLE {
        let ctx = unsafe { &mut SCHEDULER_CTX };
        if let Some(proc) = find_next_available_proc_id(ctx) {
            crate::ktrace("found a suitable task, changing");
            switch_to(ctx, proc, true);
        } else {
            crate::ktrace("no ready task, idle continues");
            // 4ms
            Arch::set_timer(Arch::nanos_to_ticks(4_000_000) + current_ticks);
        }
    } else {
        // 32ms
        if Arch::ticks_to_nanos(current_ticks)
            - Arch::ticks_to_nanos(get_currently_running_process().ticks_at_started_running)
            >= PER_PROCESS_TIME_SLICE_NANOS
        {
            ktrace("time is up, we are scheduling");
            get_currently_running_process_mut().state = task::ProcessState::Ready;
            task::schedule(true);
        } else {
            // 4ms
            Arch::set_timer(Arch::nanos_to_ticks(4_000_000) + current_ticks);
        }
    }
}

pub fn schedule(reset_timer: bool) {
    let mut buf = [0; 20];

    let ctx = unsafe { &mut SCHEDULER_CTX };

    match find_next_available_proc_id(ctx) {
        Some(TASK_PID_REAPER) => {
            let current_process = task::get_process_at_mut(ctx.current_running_proc_idx);
            let reaper_process = task::get_process_at_mut(TASK_PID_REAPER);
            reaper_process.state = task::ProcessState::Blocked;
            Arch::set_kernel_sp(0);
            Arch::switch(
                (&mut current_process.context) as *mut ContextOf<Arch>,
                &reaper_process.context as *const ContextOf<Arch>,
            );
        }
        Some(next_proc_id) => {
            kdebug("current proc: \n\t");
            kdebug(u64_to_str(ctx.current_running_proc_idx as u64, &mut buf));

            kdebug("switching to: \n\t");
            kdebug(u64_to_str(next_proc_id as u64, &mut buf));

            switch_to(ctx, next_proc_id, reset_timer);
        }
        None => {
            let idle_process = task::get_process_at_mut(TASK_PID_IDLE);
            idle_process.ticks_at_started_running = Arch::read_current_time();
            if ctx.current_running_proc_idx == TASK_PID_IDLE {
                return;
            }
            let current_process = task::get_process_at_mut(ctx.current_running_proc_idx);
            ctx.current_running_proc_idx = TASK_PID_IDLE;
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

fn switch_to(ctx: &mut Scheduler, process_id: usize, reset_timer: bool) {
    {
        let next_process = task::get_process_at_mut(process_id);

        Arch::set_root_page_table(next_process.address_space.root_pt);

        next_process.ticks_at_started_running = Arch::read_current_time();

        if reset_timer {
            // 4ms
            Arch::set_timer(
                Arch::nanos_to_ticks(4_000_000) + next_process.ticks_at_started_running,
            );
        }

        next_process.state = crate::ProcessState::Running;

        if ctx.current_running_proc_idx == process_id {
            return;
        }
    }

    let current_process = task::get_process_at_mut(ctx.current_running_proc_idx);

    ctx.current_running_proc_idx = process_id;

    Arch::switch(
        (&mut current_process.context) as *mut ContextOf<Arch>,
        (&task::get_process_at(process_id).context) as *const ContextOf<Arch>,
    );
}

fn find_next_available_proc_id(ctx: &Scheduler) -> Option<usize> {
    let time = Arch::read_current_time();

    for proc in task::iterate_process_table_mut(ctx.current_running_proc_idx) {
        if proc.pid == TASK_PID_IDLE {
            continue;
        }
        match proc.state {
            crate::task::ProcessState::Sleeping => {
                if time > proc.wake_up_at {
                    proc.wake_up_at = 0;
                    return Some(proc.pid);
                }
            }
            crate::task::ProcessState::Ready => {
                crate::ktrace("going to run: \n");
                let mut buf = [0; 20];
                crate::ktrace(u64_to_str(proc.pid as u64, &mut buf));
                return Some(proc.pid);
            }
            _ => {}
        }
    }

    None
}
