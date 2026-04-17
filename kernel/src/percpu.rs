use core::ptr::NonNull;

use ksync::SpinLock;

use crate::{sched::PerCoreScheduler, task::Task};

pub const MAX_CORES: usize = 16;

static CORES: CoreTable = CoreTable(heapless::Vec::new());

#[repr(C)]
pub struct PerCoreContext {
    pub core_id: usize,
    pub scheduler: SpinLock<PerCoreScheduler>,
    pub currently_running_task: NonNull<Task>,
    pub idle_task: NonNull<Task>,
    pub reaper_task: NonNull<Task>,
}

// TODO(aeryz): Storing pointers (that can possibly be *mut) in the PerCpuContext
// unimpls the `Send` which results in the fact that we cannot put it in a `SpinLock`.
// The fact that we can have mutable reference to the `PerCpuContext` and `CORES` at the
// same time is very dangerous. This is probably not the correct abstraction to go. And
// we really need to think more about this.
unsafe impl Send for PerCoreContext {}

struct CoreTable(heapless::Vec<PerCoreContext, MAX_CORES>);

/// SAFETY:
/// CoreTable is a fixed table that will be initialized with the cores once and won't be mutated
/// anymore.
unsafe impl Send for CoreTable {}
unsafe impl Sync for CoreTable {}

pub fn get_core_count() -> usize {
    CORES.0.len()
}

pub fn get_core<'a>(idx: usize) -> &'a PerCoreContext {
    &CORES.0[idx]
}
