use crate::{
    arch::{Context, TrapFrame},
    mm::VirtualAddress,
};

pub const PROCESS_TEXT_ADDRESS: VirtualAddress =
    unsafe { VirtualAddress::from_raw_unchecked(0x1_0000) };
pub const PROCESS_STACK_ADDRESS: VirtualAddress =
    unsafe { VirtualAddress::from_raw_unchecked(0x0000_0000_3fff_3fa0) };

#[derive(Clone)]
#[repr(C)]
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
    /// The tick count at when the process started running
    pub ticks_at_started_running: u64,
    /// The current state of the process
    pub state: ProcessState,
    /// Wake up time in ticks
    pub wake_up_at: u64,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub enum ProcessState {
    Sleeping,
    Running,
    Ready,
    Blocked,
}
