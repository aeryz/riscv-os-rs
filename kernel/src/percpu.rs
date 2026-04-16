use core::ptr::NonNull;

use ksync::SpinLock;

use crate::{sched::PerCoreScheduler, task::Task};

#[repr(C)]
pub struct PerCoreContext {
    pub scheduler: SpinLock<PerCoreScheduler>,
    pub currently_running_task: NonNull<Task>,
    pub idle_task: NonNull<Task>,
    pub reaper_task: NonNull<Task>,
}
