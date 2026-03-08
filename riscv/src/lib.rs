#![no_std]

use crate::registers::Satp;
use core::arch::asm;

pub mod registers;

pub fn clear_tlb() {
    unsafe { asm!("sfence.vma x0, x0", options(nostack, preserves_flags)) }
}

pub fn write_satp(satp: Satp) {
    satp.write();
    clear_tlb();
}

pub fn sret(user_sp: u64) -> ! {
    unsafe {
        asm!(
            "mv sp, {}",
            "sret",
            in(reg) user_sp,
            options(noreturn, nostack, preserves_flags)
        )
    }
}

pub fn mret() -> ! {
    unsafe { asm!("mret", options(noreturn, nostack, preserves_flags)) }
}
