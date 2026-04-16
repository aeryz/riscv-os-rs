use core::arch::asm;

use crate::{kmain, mm};

#[unsafe(no_mangle)]
pub extern "C" fn bootentry(hart_id: usize, dtb_pa: usize) -> ! {
    mm::early_init();

    unsafe {
        asm!(
            "li t0, {kernel_offset}",
            "add t0, t0, {}",
            "mv a0, {}",
            "mv a1, {}",
            "jr t0",
            in(reg) kmain as *const () as u64,
            in(reg) hart_id,
            in(reg) dtb_pa,
            kernel_offset = const (mm::KERNEL_IMAGE_START_VA.raw() - mm::KERNEL_IMAGE_START_PA.raw()),
            options(noreturn, nostack, preserves_flags))
    }
}
