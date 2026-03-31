#[cfg(feature = "riscv")]
mod riscv;

#[cfg(feature = "riscv")]
pub use riscv::*;

/// Defines all the architecture-dependent functionality.
pub trait Architecture {
    type Context: Context<Self>;
    type MemoryModel: MemoryModel;
    type TrapFrame: TrapFrame<Self>;

    fn set_timer(time: usize);

    fn read_current_time() -> usize;

    fn switch(from: *mut Self::Context, to: *const Self::Context);

    fn set_kernel_sp(value: usize);

    fn trap_resume_ptr() -> *const ();
}

pub type VirtualAddressOf<Arch> =
    <<Arch as Architecture>::MemoryModel as MemoryModel>::VirtualAddress;
pub type PhysicalAddressOf<Arch> =
    <<Arch as Architecture>::MemoryModel as MemoryModel>::PhysicalAddress;
pub type ContextOf<Arch> = <Arch as Architecture>::Context;
pub type TrapFrameOf<Arch> = <Arch as Architecture>::TrapFrame;

pub trait MemoryModel {
    type PhysicalAddress: Into<usize>;

    type VirtualAddress: Into<usize>;

    fn set_root_page_table(pa: Self::PhysicalAddress);
}

pub trait Context<A: Architecture + ?Sized> {
    fn initialize(entry: VirtualAddressOf<A>, kernel_sp: VirtualAddressOf<A>) -> Self;
}

pub trait TrapFrame<A: Architecture + ?Sized> {
    fn initialize(instruction_ptr: VirtualAddressOf<A>, stack_ptr: VirtualAddressOf<A>) -> Self;
}
