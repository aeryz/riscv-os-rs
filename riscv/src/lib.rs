#![no_std]

use crate::registers::Satp;
use core::arch::asm;

pub mod registers;
pub mod sbi;

pub fn clear_tlb() {
    unsafe { asm!("sfence.vma x0, x0", options(nostack, preserves_flags)) }
}

pub fn write_satp_tlb_flush(satp: Satp) {
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

#[inline(always)]
pub fn const_add_to_sp<const N: usize>() {
    unsafe {
        core::arch::asm!(
            "li t0, {kernel_offset}",
            "add sp, sp, t0",
            kernel_offset = const N,
            options(nostack, preserves_flags),
        );
    }
}
