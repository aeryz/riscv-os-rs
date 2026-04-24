use core::ptr::NonNull;

use alloc::vec::Vec;
use ksync::SpinLock;

use crate::task::Task;

static TASK_TABLE: TaskTable = TaskTable(SpinLock::new(Vec::new()));

pub fn add_task(task: Task) -> NonNull<Task> {
    let mut table = TASK_TABLE.0.lock();
    let len = table.len();
    table.push(task);

    NonNull::new(&mut table[len] as *mut Task).expect("task is nonnull")
}

struct TaskTable(SpinLock<Vec<Task>>);

unsafe impl Send for TaskTable {}
unsafe impl Sync for TaskTable {}
