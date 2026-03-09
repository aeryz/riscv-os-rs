mod macros;
mod medeleg;
mod mideleg;
mod mstatus;
mod pmpcfg;
mod satp;
mod sstatus;

pub use medeleg::*;
pub use mideleg::*;
pub use mstatus::*;
pub use pmpcfg::*;
pub use satp::*;
pub use sstatus::*;

use crate::def_impl_control_register;

def_impl_control_register!(Sepc, sepc);
def_impl_control_register!(Stvec, stvec);
def_impl_control_register!(Sscratch, sscratch);
def_impl_control_register!(Mepc, mepc);
def_impl_control_register!(Pmpaddr0, pmpaddr0);
def_impl_control_register!(Mtvec, mtvec);
