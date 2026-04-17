use core::ptr::NonNull;

use ksync::SpinLock;

use crate::{
    Arch,
    arch::{Architecture, ContextOf},
    mm,
    percpu::{self, PerCoreContext},
    task::{Task, TaskState},
};

pub const MAX_TASK_COUNT: usize = 32;

static SCHEDULER_CTX: SpinLock<GlobalScheduler> = SpinLock::new(GlobalScheduler {
    last_rq_hart_idx: 0,
});

pub struct GlobalScheduler {
    last_rq_hart_idx: usize,
}

#[repr(C)]
pub struct PerCoreScheduler {
    /// The list of ready tasks.
    /// The list of the runnable tasks for this hart.
    runqueue: heapless::Deque<NonNull<Task>, MAX_TASK_COUNT>,
    /// The time when the currently running process started running.
    last_entrance_time: usize,
}

/// Schedules a task
///
/// This does not guarantee that the currently running task will change. If there are no runnable
/// tasks and the currently running task is `Ready`, it will continue to run.
pub fn schedule() {
    let ctx = unsafe {
        Arch::load_this_cpu_ctx::<PerCoreContext>()
            .as_mut()
            .expect("expected a valid reference to the per-CPU context")
    };

    let mut sched = ctx.scheduler.lock();
    match sched.runqueue.pop_front() {
        Some(mut task) => {
            let mut current_task = ctx.currently_running_task;

            let new_task = unsafe { task.as_mut() };
            new_task.state = TaskState::Running;

            ctx.currently_running_task = NonNull::new(new_task).expect("the task is nonnull");
            sched.runqueue.push_back(current_task);

            Arch::switch_to(
                unsafe { (&mut current_task.as_mut().context) as *mut ContextOf<Arch> },
                (&new_task.context) as *const ContextOf<Arch>,
            );
        }
        None => {
            let current_task = unsafe { ctx.currently_running_task.as_mut() };
            // If there are no tasks that we can run and the currently running task can continue to be run,
            // we just run it. This also covers if the current_task is the idle task.
            if current_task.state == TaskState::Ready {
                // TODO(aeryz): set last entrance time??
                current_task.state = TaskState::Running;
                return;
            }

            ctx.currently_running_task = ctx.idle_task;
            let idle_task = unsafe { ctx.idle_task.as_mut() };
            idle_task.state = TaskState::Running;

            Arch::switch_to(
                (&mut current_task.context) as *mut ContextOf<Arch>,
                (&idle_task.context) as *const ContextOf<Arch>,
            );
        }
    }
}

/// Enqueues a new task to one of the runqueues.
///
/// The runqueue selection is round robin as well.
pub fn enqueue_new_task(task: NonNull<Task>) {
    let idx = {
        let mut scheduler_ctx = SCHEDULER_CTX.lock();

        if scheduler_ctx.last_rq_hart_idx >= percpu::get_core_count() {
            scheduler_ctx.last_rq_hart_idx = 0;
        } else {
            scheduler_ctx.last_rq_hart_idx += 1;
        }

        scheduler_ctx.last_rq_hart_idx
    };

    percpu::get_core(idx)
        .scheduler
        .lock()
        .runqueue
        .push_back(task);
}
