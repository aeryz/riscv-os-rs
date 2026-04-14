use core::arch::asm;

use crate::{kdebug, kmain, mm, usize_to_str, usize_to_str_hex};

#[unsafe(no_mangle)]
pub extern "C" fn bootentry(hart_id: usize, dtb_pa: usize) -> ! {
    let mut buf = [0; 20];
    kdebug("hart id: ");
    kdebug(usize_to_str(hart_id, &mut buf));
    kdebug("dtb pa: ");
    kdebug(usize_to_str_hex(dtb_pa, &mut buf));

    let magic = u32::from_be(unsafe { *(dtb_pa as *const u32) });
    kdebug("magic: ");
    kdebug(usize_to_str_hex(magic as usize, &mut buf));

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
