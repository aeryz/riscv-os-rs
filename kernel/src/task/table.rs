use core::ptr::NonNull;

use ksync::SpinLock;

use crate::task::Task;

const MAX_TASK: usize = 32;

static TASK_TABLE: TaskTable = TaskTable(SpinLock::new(heapless::Vec::new()));

// TODO(aeryz): this sleep table can be smarter. Instead of pushing to the vec, we can
// insert it in a sorted way and pop later.
static SLEEP_TABLE: SleepTable = SleepTable(SpinLock::new(heapless::Vec::new()));

pub fn add_task(task: Task) -> NonNull<Task> {
    let mut table = TASK_TABLE.0.lock();
    let len = table.len();
    unsafe { table.push_unchecked(task) };

    NonNull::new(&mut table[len] as *mut Task).expect("task is nonnull")
}

pub fn add_sleeping_task(task: NonNull<Task>) {
    let mut table = SLEEP_TABLE.0.lock();
    table
        .push(task)
        .expect("we ran out of slots in the task table");
}

struct SleepTable(SpinLock<heapless::Vec<NonNull<Task>, MAX_TASK>>);

unsafe impl Send for SleepTable {}
unsafe impl Sync for SleepTable {}

struct TaskTable(SpinLock<heapless::Vec<Task, MAX_TASK>>);

unsafe impl Send for TaskTable {}
unsafe impl Sync for TaskTable {}
