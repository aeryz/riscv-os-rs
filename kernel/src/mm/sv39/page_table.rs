use crate::mm::{self, PageTableEntry, PteFlags, VirtualAddress};

use super::PhysicalAddress;

#[repr(C, align(4096))]
pub struct PageTable([PageTableEntry; 512]);

impl PageTable {
    pub const fn empty() -> Self {
        PageTable([PageTableEntry::empty(); 512])
    }

    pub const fn set_entry(&mut self, idx: usize, entry: PageTableEntry) {
        self.0[idx] = entry;
    }

    /// Map the `va` to `pa`.
    ///
    /// This only meant to operate when the virtual memory is not enabled.
    pub fn map_vm_early(&mut self, va: VirtualAddress, pa: PhysicalAddress, flags: PteFlags) {
        let l2_entry = &mut self.0[va.vpn_2()];

        let l1_page_table = Self::get_or_create_next_table(l2_entry);

        let l1_entry = unsafe { (*l1_page_table).0.get_unchecked_mut(va.vpn_1()) };
        let l0_page_table = Self::get_or_create_next_table(l1_entry);

        let l0_entry = unsafe { (*l0_page_table).0.get_unchecked_mut(va.vpn_0()) };
        if !l0_entry.is_valid() {
            *l0_entry = l0_entry
                .set_flags(flags | PteFlags::V | PteFlags::A | PteFlags::D)
                .set_physical_address(pa);
        }
    }

    pub fn map_user_memory(&mut self, va: VirtualAddress, pa: PhysicalAddress, flags: PteFlags) {
        let l2_entry = &mut self.0[va.vpn_2()];
        let l1_page_table: *mut PageTable = if !l2_entry.is_valid() {
            let pa = mm::alloc().unwrap();
            let va =
                VirtualAddress::from_raw(pa.raw() + mm::KERNEL_DIRECT_MAPPING_BASE.raw()).unwrap();
            let page_table_ptr = va.as_ptr_mut();
            unsafe {
                *page_table_ptr = PageTable::empty();
            }
            *l2_entry = PageTableEntry::new_valid().set_physical_address(pa);
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
            *l1_entry = PageTableEntry::new_valid().set_physical_address(pa);
            page_table_ptr
        } else {
            (l1_entry.physical_address().raw() + mm::KERNEL_DIRECT_MAPPING_BASE.raw())
                as *mut PageTable
        };

        let l0_entry = unsafe { (*l0_page_table).0.get_unchecked_mut(va.vpn_0()) };
        if !l0_entry.is_valid() {
            *l0_entry = l0_entry
                .set_flags(flags | PteFlags::V | PteFlags::A | PteFlags::D)
                .set_physical_address(pa);
        }
    }

    fn get_or_create_next_table(pte: &mut PageTableEntry) -> *mut PageTable {
        if pte.is_valid() {
            return pte.physical_address().as_ptr_mut();
        }

        let pa = mm::alloc().unwrap();
        let page_table_ptr = pa.as_ptr_mut();
        unsafe {
            *page_table_ptr = PageTable::empty();
        }
        *pte = PageTableEntry::new_valid().set_physical_address(pa);
        page_table_ptr
    }
}
