use core::ptr::NonNull;

use ksync::SpinLock;

use crate::{
    Arch,
    arch::Architecture,
    mm,
    percpu::PerCoreContext,
    task::{Task, TaskState},
};

pub const MAX_TASK_COUNT: usize = 32;

#[repr(C)]
pub struct PerCoreScheduler {
    /// The list of ready tasks.
    /// The list of the runnable tasks for this hart.
    runqueue: heapless::Deque<NonNull<Task>, MAX_TASK_COUNT>,
    /// The time when the currently running process started running.
    last_entrance_time: usize,
}

pub fn schedule() {
    let ctx = unsafe {
        Arch::load_this_cpu_ctx::<PerCoreContext>()
            .as_mut()
            .expect("expected a valid reference to the per-CPU context")
    };

    let mut sched = ctx.scheduler.lock();
    match sched.runqueue.pop_front() {
        Some(mut task) => {
            let new_task = unsafe { task.as_mut() };
            new_task.state = TaskState::Running;

            // TODO(aeryz): switch to this task
        }
        None => {
            let current_task = unsafe { ctx.currently_running_task.as_mut() };
            // If there are no tasks that we can run and the currently running task can continue to be run,
            // we just run it. This also covers if the current_task is the idle task.
            if current_task.state == TaskState::Ready {
                // TODO(aeryz): set last entrance time??
                return;
            }

            // TODO(aeryz): switch to the idle task
        }
    }
}
