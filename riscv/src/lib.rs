#![no_std]

use core::arch::asm;
pub mod registers;

pub fn clear_tlb() {
    unsafe { asm!("sfence.vma x0, x0", options(nostack, preserves_flags)) }
}
