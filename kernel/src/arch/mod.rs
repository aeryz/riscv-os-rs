#![allow(unused)]

#[cfg(feature = "riscv-sbi")]
mod riscv;

use core::ptr::NonNull;

#[cfg(feature = "riscv-sbi")]
pub use riscv::*;

/// Defines all the architecture-dependent functionality.
pub trait Architecture {
    const CPU_HERTZ: usize;

    type MemoryModel: MemoryModel;

    #[inline(always)]
    fn bump_sp(sp: usize);

    /// Loads the pointer to the current CPU context.
    ///
    /// SAFETY:
    /// - It's totally kernel's responsibility to properly set the CPU context.
    #[inline(always)]
    fn load_this_cpu_ctx<T>() -> *mut T;

    /// Reads the current time
    fn read_current_time() -> usize;
}

pub type VirtualAddressOf<Arch> =
    <<Arch as Architecture>::MemoryModel as MemoryModel>::VirtualAddress;
pub type PhysicalAddressOf<Arch> =
    <<Arch as Architecture>::MemoryModel as MemoryModel>::PhysicalAddress;

pub trait MemoryModel {
    type PhysicalAddress: Into<usize>;

    type VirtualAddress: Into<usize>;

    fn set_root_page_table(pa: Self::PhysicalAddress);

    fn get_root_page_table() -> usize;
}
