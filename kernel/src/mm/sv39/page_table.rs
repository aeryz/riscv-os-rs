use crate::{
    kdebug,
    mm::{self, PageTableEntry, VirtualAddress},
};

use super::PhysicalAddress;

#[repr(C, align(4096))]
pub struct PageTable([PageTableEntry; 512]);

pub enum Perm {
    Read,
    Write,
    Execute,
    All,
}

impl PageTable {
    pub const fn empty() -> Self {
        PageTable([PageTableEntry::empty(); 512])
    }

    pub const fn set_entry(&mut self, idx: usize, entry: PageTableEntry) {
        self.0[idx] = entry;
    }

    pub fn create_identity_mapped_page(&mut self, addr: PhysicalAddress, perm: Perm) {
        let mut buf = [0; 20];
        kdebug(b"entered identity map\n".as_slice());

        let va = addr.to_identity_mapped_va().unwrap();

        kdebug(b"vpn_2: ".as_slice());
        kdebug(crate::u64_to_str(va.vpn_2() as u64, &mut buf));
        kdebug(b"vpn_1: ".as_slice());
        kdebug(crate::u64_to_str(va.vpn_1() as u64, &mut buf));
        kdebug(b"vpn_0: ".as_slice());
        kdebug(crate::u64_to_str(va.vpn_0() as u64, &mut buf));

        let l2_entry = &mut self.0[va.vpn_2()];
        let l1_page_table: *mut PageTable = if !l2_entry.is_valid() {
            let pa = mm::alloc().unwrap();
            let page_table_ptr = pa.as_ptr_mut();
            kdebug("ptr:");
            kdebug(crate::u64_to_str_hex(page_table_ptr as u64, &mut [0; 20]));
            unsafe {
                *page_table_ptr = PageTable::empty();
            }
            *l2_entry = PageTableEntry::new_pointer().set_physical_address(pa);
            page_table_ptr
        } else {
            l2_entry.physical_address().as_ptr_mut()
        };

        let l1_entry = unsafe { (*l1_page_table).0.get_unchecked_mut(va.vpn_1()) };
        let l0_page_table: *mut PageTable = if !l1_entry.is_valid() {
            let pa = mm::alloc().unwrap();
            let page_table_ptr = pa.as_ptr_mut();
            unsafe {
                *page_table_ptr = PageTable::empty();
            }
            *l1_entry = PageTableEntry::new_pointer().set_physical_address(pa);
            page_table_ptr
        } else {
            l1_entry.physical_address().as_ptr_mut()
        };

        let l0_entry = unsafe { (*l0_page_table).0.get_unchecked_mut(va.vpn_0()) };
        if !l0_entry.is_valid() {
            *l0_entry = match perm {
                Perm::Read => l0_entry.set_readable(),
                Perm::Write => l0_entry.set_writable(),
                Perm::Execute => l0_entry.set_executable(),
                Perm::All => l0_entry.set_rwx(),
            }
            .set_valid()
            .set_physical_address(addr)
            .set_dirty()
            .set_accessed();
        }
    }

    pub fn map_user_memory(
        &mut self,
        va: VirtualAddress,
        pa: PhysicalAddress,
        perm: Perm,
        is_user: bool,
    ) {
        let l2_entry = &mut self.0[va.vpn_2()];
        let l1_page_table: *mut PageTable = if !l2_entry.is_valid() {
            let pa = mm::alloc().unwrap();
            let va =
                VirtualAddress::from_raw(pa.raw() + mm::KERNEL_DIRECT_MAPPING_BASE.raw()).unwrap();
            let page_table_ptr = va.as_ptr_mut();
            unsafe {
                *page_table_ptr = PageTable::empty();
            }
            *l2_entry = PageTableEntry::new_pointer().set_physical_address(pa);
            page_table_ptr
        } else {
            (l2_entry.physical_address().raw() + mm::KERNEL_DIRECT_MAPPING_BASE.raw())
                as *mut PageTable
        };

        let l1_entry = unsafe { (*l1_page_table).0.get_unchecked_mut(va.vpn_1()) };
        let l0_page_table: *mut PageTable = if !l1_entry.is_valid() {
            let pa = mm::alloc().unwrap();
            let va =
                VirtualAddress::from_raw(pa.raw() + mm::KERNEL_DIRECT_MAPPING_BASE.raw()).unwrap();
            let page_table_ptr = va.as_ptr_mut();
            unsafe {
                *page_table_ptr = PageTable::empty();
            }
            *l1_entry = PageTableEntry::new_pointer().set_physical_address(pa);
            page_table_ptr
        } else {
            (l1_entry.physical_address().raw() + mm::KERNEL_DIRECT_MAPPING_BASE.raw())
                as *mut PageTable
        };

        let l0_entry = unsafe { (*l0_page_table).0.get_unchecked_mut(va.vpn_0()) };
        if !l0_entry.is_valid() {
            *l0_entry = match perm {
                Perm::Read => l0_entry.set_readable(),
                Perm::Write => l0_entry.set_writable(),
                Perm::Execute => l0_entry.set_executable(),
                Perm::All => l0_entry.set_rwx(),
            }
            .set_valid()
            .set_physical_address(pa)
            .set_dirty()
            .set_accessed();

            if is_user {
                *l0_entry = l0_entry.set_user_accessible();
            }
        }
    }
}
