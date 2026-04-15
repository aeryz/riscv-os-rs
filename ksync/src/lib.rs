#![no_std]

mod rw_lock;
mod spin_lock;

pub use rw_lock::*;
pub use spin_lock::*;
