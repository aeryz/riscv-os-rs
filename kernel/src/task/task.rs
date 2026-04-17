use core::ptr::NonNull;

use crate::{
    Arch,
    arch::{ContextOf, VirtualAddressOf},
    mm,
    task::{AddressSpace, Pid, TaskState},
};

#[repr(C)]
#[derive(Clone)]
pub struct Task {
    /// Process ID
    pub pid: Pid,
    /// Kernel stack pointer
    pub kernel_sp: VirtualAddressOf<Arch>,
    /// Pointer to the context
    pub context: NonNull<ContextOf<Arch>>,
    /// The current state of the process
    pub state: TaskState,
    /// Wake up time in ticks
    pub wake_up_at: usize,
    // TODO(aeryz): We can consider putting this exit code into the relevant state enum
    /// Process exit code
    pub exit_code: i32,
    /// Address space
    pub address_space: AddressSpace,
}
