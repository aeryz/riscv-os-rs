mod macros;
mod mcounteren;
mod medeleg;
mod menvcfg;
mod mideleg;
mod mstatus;
mod pmpcfg;
mod satp;
mod sie;
mod sstatus;

pub use mcounteren::*;
pub use medeleg::*;
pub use menvcfg::*;
pub use mideleg::*;
pub use mstatus::*;
pub use pmpcfg::*;
pub use satp::*;
pub use sie::*;
pub use sstatus::*;

use crate::def_impl_control_register;

def_impl_control_register!(Sepc, sepc);
def_impl_control_register!(Stvec, stvec);
def_impl_control_register!(Sscratch, sscratch);
def_impl_control_register!(Mepc, mepc);
def_impl_control_register!(Pmpaddr0, pmpaddr0);
def_impl_control_register!(Mtvec, mtvec);
def_impl_control_register!(Time, time);
def_impl_control_register!(Stimecmp, stimecmp);
def_impl_control_register!(Stval, stval);
def_impl_control_register!(Scause, scause);
