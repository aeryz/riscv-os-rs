use riscv::registers::{Satp, SatpMode};

use crate::{
    arch::mmu::{PageTable, PageTableEntry, PhysicalAddress, PteFlags},
    helper::GB,
    mm::{self, allocator},
};

// TODO(aeryz): add spinlock:
// But I still need to confirm how a lock would actually work here. Because we cannot
// prohibit the hardware from accessing here while modifying this table.
static mut KERNEL_ROOT_PAGE_TABLE: PageTable = PageTable::empty();

unsafe extern "C" {
    static __text_start: u8;
    static __text_end: u8;
    static __kernel_end: u8;
}

/// Saves the kernel root table
#[inline(never)]
pub fn init() {
    // TODO(aeryz): We want to have a separate spot for the allocatable memory.
    let memory_start =
        unsafe { PhysicalAddress::from_raw_unchecked(&__kernel_end as *const u8 as u64) };
    allocator::init(memory_start);

    let text_end = unsafe { &__text_end as *const u8 as u64 };
    unsafe {
        let mut text_start = PhysicalAddress::from_raw_unchecked(&__text_start as *const u8 as u64);
        let n_text_pages = (text_end - text_start.raw()) / 4096 + 1;
        kvm_full_map(&mut KERNEL_ROOT_PAGE_TABLE);
        crate::kdebug("kvm full mapped \n");
        for _ in 0..n_text_pages {
            KERNEL_ROOT_PAGE_TABLE.map_vm_early(
                text_start.to_identical_va().unwrap(),
                text_start,
                PteFlags::RWX,
            );
            text_start = PhysicalAddress::from_raw_unchecked(text_start.raw() + 0x1000);
        }
    }

    crate::kdebug("before satp\n");
    riscv::write_satp(
        Satp::empty()
            .set_mode(SatpMode::Sv39)
            .set_ppn((unsafe { &KERNEL_ROOT_PAGE_TABLE }) as *const PageTable as u64),
    );
    crate::kdebug("after satp\n");

    riscv::const_add_to_sp::<
        { (mm::KERNEL_IMAGE_START_VA.raw() - mm::KERNEL_IMAGE_START_PA.raw()) as usize },
    >();
}

// TODO(aeryz): make this dynamic
/// Maps the whole ram and the kernel image so that
/// the processes and the kernel can easily access pretty much anywhere
pub fn kvm_full_map(page_table: &mut PageTable) {
    let va = mm::KERNEL_DIRECT_MAPPING_BASE;

    let base_pte =
        PageTableEntry::empty().set_flags(PteFlags::V | PteFlags::RW | PteFlags::A | PteFlags::D);

    let mut i = 0;
    for p_i in va.vpn_2()..510 {
        let pa = unsafe { PhysicalAddress::from_raw_unchecked((i * GB) as u64) };
        page_table.set_entry(p_i, base_pte.clone().set_physical_address(pa));
        i += 1;
    }

    // kernel image
    // TODO(aeryz): for convenience, will just have 2 1GB RWX tables
    let mut i = 0;
    for p_i in 510..512 {
        let pa = unsafe {
            PhysicalAddress::from_raw_unchecked(
                (mm::KERNEL_IMAGE_START_PA.raw() as usize + i * GB) as u64,
            )
        };

        page_table.set_entry(
            p_i,
            base_pte
                .clone()
                .set_flags(PteFlags::RX)
                .set_physical_address(pa),
        );

        i += 1;
    }
}
