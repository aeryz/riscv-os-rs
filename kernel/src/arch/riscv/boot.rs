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

#[cfg(feature = "multi-core")]
#[unsafe(naked)]
pub extern "C" fn core_entry_trampoline() -> ! {
    core::arch::naked_asm!(
        r#"
        mv sp, a1
        ld a2, 0(sp)

        csrw satp, a2
        sfence.vma

        li t0, {kernel_offset}
        la t1, core_boot_entry
        add t0, t0, t1

        li t1, {kernel_direct_mapping_base}
        add sp, sp, t1

        jr t0
        "#,
        kernel_offset = const (mm::KERNEL_IMAGE_START_VA.raw() - mm::KERNEL_IMAGE_START_PA.raw()),
        kernel_direct_mapping_base = const (mm::KERNEL_DIRECT_MAPPING_BASE.raw()),
    )
}

#[cfg(feature = "multi-core")]
fn boot_core(core: usize) {
    let mut sp = mm::alloc_frame().unwrap().raw() + 0xff0;

    sp = sp - size_of::<usize>();

    unsafe {
        let sp_kernel_view = sp + mm::KERNEL_DIRECT_MAPPING_BASE.raw();
        *(sp_kernel_view as *mut usize) = Arch::get_root_page_table();
    }

    let ret = riscv::sbi::hart_start(
        core,
        core_entry_trampoline as *const () as usize
            - (mm::KERNEL_IMAGE_START_VA.raw() - mm::KERNEL_IMAGE_START_PA.raw()),
        sp,
    );

    if ret.error == 0 {
        log::info!("core {core} started successfully");
    } else {
        log::error!("core {core} start failure");
        panic!();
    }
}
