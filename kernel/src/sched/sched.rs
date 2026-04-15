use core::ptr::NonNull;

use ksync::SpinLock;

use crate::{mm, task::Task};

pub const MAX_TASK_COUNT: usize = 32;

/// A global task list
static RUNNABLE_TASK_LIST: SpinLock<heapless::Vec<Task, MAX_TASK_COUNT>> =
    SpinLock::new(heapless::Vec::new());

#[repr(C)]
pub struct PerHartScheduler {
    // TODO(aeryz): This max task size is tailored for having 4 cores. But this is only temporary.
    // Once we have `kmalloc`, we will introduce proper heap allocated structures with no or very large
    // boundaries.
    /// Runnable tasks that are assigned to this hart.
    runqueue: heapless::Deque<NonNull<Task>, { MAX_TASK_COUNT / 4 }>,
    /// The index of the currently running task.
    currently_running_task_idx: usize,
    /// The time when the currently running process started running.
    last_entrance_time: usize,
}
