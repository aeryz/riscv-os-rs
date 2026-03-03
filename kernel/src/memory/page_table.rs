use crate::{allocator::Allocator, debug, memory::page_table_entry::PageTableEntry};

use super::physical_address::PhysicalAddress;

pub struct PageTable([PageTableEntry; 512]);

pub enum Perm {
    Read,
    Write,
    Execute,
    All,
}

impl PageTable {
    pub fn empty() -> Self {
        PageTable([PageTableEntry::empty(); 512])
    }

    pub fn create_identity_mapped_page<const N: usize>(
        &mut self,
        addr: PhysicalAddress,
        allocator: &mut Allocator<N>,
        perm: Perm,
        is_user: bool,
    ) {
        let mut buf = [0; 20];
        debug(b"entered identity map\n".as_slice());

        let va = addr.to_identity_mapped_va().unwrap();

        debug(b"vpn_2: ".as_slice());
        debug(crate::u64_to_str(va.vpn_2() as u64, &mut buf));
        debug(b"vpn_1: ".as_slice());
        debug(crate::u64_to_str(va.vpn_1() as u64, &mut buf));
        debug(b"vpn_0: ".as_slice());
        debug(crate::u64_to_str(va.vpn_0() as u64, &mut buf));

        let l2_entry = &mut self.0[va.vpn_2()];
        let l1_page_table: *mut PageTable = if !l2_entry.is_valid() {
            debug(b"[l2_entry] is not valid\n".as_slice());
            let pa = allocator.alloc().unwrap();
            let page_table_ptr = pa.as_ptr_mut();
            unsafe {
                *page_table_ptr = PageTable::empty();
            }
            debug(b"[l2_entry] physical address".as_slice());
            debug(crate::u64_to_str(pa.raw(), &mut buf));
            *l2_entry = PageTableEntry::new_pointer().set_physical_address(pa);
            if is_user {
                *l2_entry = l2_entry.set_user_accessible();
            }
            page_table_ptr
        } else {
            debug(b"l2 entry is valid\n".as_slice());
            l2_entry.physical_address().as_ptr_mut()
        };
        let l2_entry = &mut self.0[va.vpn_2()];
        if !l2_entry.is_valid() {
            debug(b"l2 entry still not valid\n".as_slice());
        } else {
            debug(b"l2 entry finally valid\n".as_slice());
        }

        let l1_entry = unsafe { (*l1_page_table).0.get_unchecked_mut(va.vpn_1()) };
        let l0_page_table: *mut PageTable = if !l1_entry.is_valid() {
            debug(b"l1 entry is not valid\n".as_slice());
            let pa = allocator.alloc().unwrap();
            let page_table_ptr = pa.as_ptr_mut();
            unsafe {
                *page_table_ptr = PageTable::empty();
            }
            *l1_entry = PageTableEntry::new_pointer().set_physical_address(pa);
            if is_user {
                *l1_entry = l1_entry.set_user_accessible();
            }
            page_table_ptr
        } else {
            debug(b"l1 entry is valid\n".as_slice());
            l1_entry.physical_address().as_ptr_mut()
        };
        let l1_entry = unsafe { (*l1_page_table).0.get_unchecked_mut(va.vpn_1()) };
        if !l1_entry.is_valid() {
            debug(b"l1 entry still not valid\n".as_slice());
        } else {
            debug(b"l1 entry finally valid\n".as_slice());
        }

        let l0_entry = unsafe { (*l0_page_table).0.get_unchecked_mut(va.vpn_0()) };
        if !l0_entry.is_valid() {
            debug(b"l0 entry is not valid\n".as_slice());
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

            if is_user {
                *l0_entry = l0_entry.set_user_accessible();
            }
        } else {
            debug(b"l0 entry is valid\n".as_slice());
        }
    }
}
