use core::{cell::OnceCell, mem::MaybeUninit};

use crate::task::Process;

static mut PROC_TABLE: ProcessTable = ProcessTable::zeroed();

#[repr(C)]
struct ProcessTable {
    table: [Process; 128],
    head: usize,
}

impl ProcessTable {
    const fn zeroed() -> Self {
        Self {
            table: [const { Process::empty() }; 128],
            head: 0,
        }
    }

    pub fn new_process(&mut self, process: Process) {
        self.table[self.head] = process;
        self.head += 1;
    }
}

pub fn add_process(process: Process) {
    let table = unsafe { &mut PROC_TABLE };
    table.new_process(process);
}

pub fn get_process_at(index: usize) -> &'static Process {
    let table = unsafe { &PROC_TABLE };
    debug_assert!(index <= table.head);

    &table.table[index]
}

pub fn get_process_at_mut(index: usize) -> &'static mut Process {
    let table = unsafe { &mut PROC_TABLE };
    debug_assert!(index <= table.head);

    &mut table.table[index]
}

pub fn iterate_process_table_mut(start_idx: usize) -> impl Iterator<Item = &'static mut Process> {
    let table = unsafe { &mut PROC_TABLE };

    let (left, right) = table.table.split_at_mut(start_idx);

    left.iter_mut().chain(right)
}
