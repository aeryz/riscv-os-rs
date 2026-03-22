use riscv::registers::{Satp, SatpMode};

use crate::mm::{self, PageTable, PageTableEntry, PhysicalAddress, allocator};

static mut KERNEL_ROOT_PAGE_TABLE: PageTable = PageTable::empty();

unsafe extern "C" {
    static __text_start: u8;
    static __text_end: u8;
    static __kernel_end: u8;
}

/// Saves the kernel root table
pub fn init() {
    let memory_start =
        unsafe { PhysicalAddress::from_raw_unchecked(&__kernel_end as *const u8 as u64) };
    allocator::init(memory_start);

    let text_end = unsafe { &__text_end as *const u8 as u64 };
    unsafe {
        let mut text_start = PhysicalAddress::from_raw_unchecked(&__text_start as *const u8 as u64);
        let n_text_pages = (text_end - text_start.raw()) / 4096 + 1;
        kvm_full_map(&mut KERNEL_ROOT_PAGE_TABLE);
        crate::kdebug(b"kvm full mapped \n");
        for _ in 0..n_text_pages {
            KERNEL_ROOT_PAGE_TABLE.create_identity_mapped_page(text_start, super::Perm::Execute);
            text_start = PhysicalAddress::from_raw_unchecked(text_start.raw() + 0x1000);
        }
    }

    riscv::write_satp(
        Satp::empty()
            .set_mode(SatpMode::Sv39)
            .set_ppn((unsafe { &KERNEL_ROOT_PAGE_TABLE }) as *const PageTable as u64),
    );
    unsafe {
        core::arch::asm!(
            "li t0, {kernel_offset}",
            "add sp, sp, t0",
            kernel_offset = const (mm::KERNEL_IMAGE_START_VA.raw() - mm::KERNEL_IMAGE_START_PA.raw()),
            options(nostack, preserves_flags),
        );
    }
}

// TODO: make this dynamic
/// Maps the whole ram and the kernel image so that
/// the processes and the kernel can easily access pretty much anywhere
pub fn kvm_full_map(page_table: &mut PageTable) {
    let va = mm::KERNEL_DIRECT_MAPPING_BASE;
    const GB: u64 = 1024 * 1024 * 1024;

    let base_pte = PageTableEntry::empty()
        .set_valid()
        .set_writable()
        .set_accessed()
        .set_dirty();

    let mut i = 0;
    for p_i in va.vpn_2()..510 {
        let pa = unsafe { PhysicalAddress::from_raw_unchecked(i as u64 * GB) };
        page_table.set_entry(p_i, base_pte.clone().set_physical_address(pa));
        i += 1;
    }

    // kernel image
    // TODO: for convenience, will just have 2 1GB RWX tables
    let mut i = 0;
    for p_i in 510..512 {
        let pa = unsafe {
            PhysicalAddress::from_raw_unchecked(mm::KERNEL_IMAGE_START_PA.raw() + i as u64 * GB)
        };
        page_table.set_entry(
            p_i,
            base_pte.clone().set_executable().set_physical_address(pa),
        );
        i += 1;
    }
}
