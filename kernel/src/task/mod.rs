mod process;
mod process_table;
mod schedule;

pub use process::*;
pub use process_table::*;
pub use schedule::*;

pub const TASK_PID_IDLE: usize = 0;
pub const TASK_PID_REAPER: usize = 1;
