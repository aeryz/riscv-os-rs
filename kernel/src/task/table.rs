use core::ptr::NonNull;

use ksync::SpinLock;

use crate::task::Task;

const MAX_TASK: usize = 32;

static TASK_TABLE: TaskTable = TaskTable(SpinLock::new(heapless::Vec::new()));

pub fn add_task(task: Task) -> NonNull<Task> {
    let mut table = TASK_TABLE.0.lock();
    let len = table.len();
    unsafe { table.push_unchecked(task) };

    NonNull::new(&mut table[len] as *mut Task).expect("task is nonnull")
}

struct TaskTable(SpinLock<heapless::Vec<Task, MAX_TASK>>);

unsafe impl Send for TaskTable {}
unsafe impl Sync for TaskTable {}
