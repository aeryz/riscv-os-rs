use crate::{allocator::Allocator, debug, page_table_entry::PageTableEntry};

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
        addr: u64,
        allocator: &mut Allocator<N>,
        perm: Perm,
        is_user: bool,
    ) {
        debug(b"entered identity map\n".as_slice());
        let vpn_2 = (addr >> 30) & ((1 << 9) - 1);
        let vpn_1 = (addr >> 21) & ((1 << 9) - 1);
        let vpn_0 = (addr >> 12) & ((1 << 9) - 1);

        let mut buf = [0; 20];
        debug(b"vpn_2: ".as_slice());
        debug(crate::u64_to_str(vpn_2, &mut buf));
        let mut buf = [0; 20];
        debug(b"vpn_1: ".as_slice());
        debug(crate::u64_to_str(vpn_1, &mut buf));
        let mut buf = [0; 20];
        debug(b"vpn_0: ".as_slice());
        debug(crate::u64_to_str(vpn_0, &mut buf));

        let l2_entry = &mut self.0[vpn_2 as usize];
        let l1_page_table: *mut PageTable = if !l2_entry.is_valid() {
            debug(b"[l2_entry] is not valid\n".as_slice());
            let pa = allocator.alloc().unwrap();
            let page_table_ptr = pa as *mut PageTable;
            unsafe {
                *page_table_ptr = PageTable::empty();
            }
            *l2_entry = PageTableEntry::new_pointer().set_physical_address(pa);
            if is_user {
                *l2_entry = l2_entry.set_user_accessible();
            }
            page_table_ptr
        } else {
            debug(b"l2 entry is valid\n".as_slice());
            l2_entry.physical_address() as *mut PageTable
        };
        let l2_entry = &mut self.0[vpn_2 as usize];
        if !l2_entry.is_valid() {
            debug(b"l2 entry still not valid\n".as_slice());
        } else {
            debug(b"l2 entry finally valid\n".as_slice());
        }

        let l1_entry = unsafe { (*l1_page_table).0.get_unchecked_mut(vpn_1 as usize) };
        let l0_page_table: *mut PageTable = if !l1_entry.is_valid() {
            debug(b"l1 entry is not valid\n".as_slice());
            let pa = allocator.alloc().unwrap();
            let page_table_ptr = pa as *mut PageTable;
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
            l1_entry.physical_address() as *mut PageTable
        };
        let l1_entry = unsafe { (*l1_page_table).0.get_unchecked_mut(vpn_1 as usize) };
        if !l1_entry.is_valid() {
            debug(b"l1 entry still not valid\n".as_slice());
        } else {
            debug(b"l1 entry finally valid\n".as_slice());
        }

        let l0_entry = unsafe { (*l0_page_table).0.get_unchecked_mut(vpn_0 as usize) };
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
