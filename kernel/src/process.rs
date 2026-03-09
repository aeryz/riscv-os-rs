pub const PROC_TEXT_VA: u64 = 0x1_0000;
pub const PROC_STACK_VA: u64 = 0x0000_0000_3fff_0fa0;
pub struct Process {
    /// Kernel stack pointer
    pub kernel_sp: u64,
    /// Root page table (PA) of this process
    pub root_table_pa: u64,
}

impl Process {}
