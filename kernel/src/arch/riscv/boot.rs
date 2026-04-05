use core::arch::asm;

use crate::helper::u64_to_str_hex;
use crate::kmain;
use crate::{helper::u64_to_str, kdebug, mm};

#[unsafe(no_mangle)]
pub extern "C" fn bootentry(hart_id: u64, dtb_pa: u64) -> ! {
    kdebug(b"hello world from kernel\n");

    let mut buf = [0; 20];
    kdebug("hart id: ");
    kdebug(u64_to_str(hart_id, &mut buf));
    kdebug("dtb pa: ");
    kdebug(u64_to_str_hex(dtb_pa, &mut buf));

    let magic = u32::from_be(unsafe { *(dtb_pa as *const u32) });
    kdebug("magic: ");
    kdebug(u64_to_str_hex(magic as u64, &mut buf));

    enter_supervisor(supervisor_main_no_virtual_memory as *const () as usize);
}

#[inline(always)]
pub fn enter_supervisor(entry: usize) -> ! {
    // `mret` will jump to `mepc` which is `entry`.
    riscv::registers::Mepc::new(entry as u64).write();
    // Enable the supervisor mode so that `mret` starts executing in the S-mode.
    riscv::registers::Mstatus::read()
        .enable_supervisor_mode()
        .write();
    // Delegate all interrupt handlings to S-mode.
    riscv::registers::Mideleg::empty().delegate_all().write();
    riscv::registers::Medeleg::empty().delegate_all().write();

    // Enable access to `rdtime` pseudo-instruction by the S-mode.
    riscv::registers::Mcounteren::empty()
        .enable_access_to_time()
        .write();

    // Enable the `stimecmp` register in S-mode.
    riscv::registers::Menvcfg::empty().enable_stimecmp().write();

    // Enable access to all memory.
    // TODO: Idk if we need to do anything here because we already do memory management in the
    // S-mode. Let's check what Linux does here.
    riscv::registers::Pmpaddr0::new(0x2fffffffffffffff).write();
    riscv::registers::Pmpcfg0::empty()
        .enable_tor()
        .set_readable()
        .set_writable()
        .set_executable()
        .write();

    // Return to address at `mepc`(entry) and start executing in the mode set in `mstatus`(S-mode)
    riscv::mret();
}

/// The entrypoint for when the initial boot phase is done and M-mode switches
/// to S-mode. Only responsibility here is to setup the kernel virtual memory,
/// and immediately switch to the higher base kernel code at [`kernel_higher_half_entry`].
/// Eg. 0x80000000 -> 0xffffffff80000000
#[unsafe(no_mangle)]
pub extern "C" fn supervisor_main_no_virtual_memory() -> ! {
    kdebug(b"hello from the supervisor\n");

    mm::init();

    unsafe {
        asm!(
            "li t0, {kernel_offset}",
            "add t0, t0, {}",
            "jr t0",
            in(reg) kmain as *const () as u64,
            kernel_offset = const (mm::KERNEL_IMAGE_START_VA.raw() - mm::KERNEL_IMAGE_START_PA.raw()), 
            options(noreturn, nostack, preserves_flags))
    }
}
