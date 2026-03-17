use crate::{context::Context, trap::TrapFrame};

pub const PROC_TEXT_VA: u64 = 0x1_0000;
pub const PROC_STACK_VA: u64 = 0x0000_0000_3fff_0fa0;

#[derive(Clone)]
pub struct Process {
    // TODO: pid is only the index in the proc table right now
    /// Process ID
    pub pid: usize,
    /// Kernel stack pointer
    pub kernel_sp: u64,
    /// Root page table (PA) of this process
    pub root_table_pa: u64,
    /// Trap frame
    pub trap_frame: *mut TrapFrame,
    /// Context
    pub context: Context,
}

impl Process {}
