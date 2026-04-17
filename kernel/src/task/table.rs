use core::ptr::NonNull;

use ksync::SpinLock;

use crate::task::Task;

const MAX_TASK: usize = 32;

pub static TASK_TABLE: SpinLock<heapless::Vec<Task, MAX_TASK>> =
    SpinLock::new(heapless::Vec::new());

pub fn add_task(task: Task) -> NonNull<Task> {
    let mut table = TASK_TABLE.lock();
    let len = table.len();
    table.push(task);
    NonNull::new(&mut table[len] as *mut Task).expect("task is nonnull")
}
