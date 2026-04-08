use core::mem::MaybeUninit;

use crate::task::Process;

static mut PROC_TABLE: ProcessTable = ProcessTable::zeroed();

#[repr(C)]
struct ProcessTable {
    table: [MaybeUninit<Process>; 128],
    head: usize,
}

impl ProcessTable {
    const fn zeroed() -> Self {
        Self {
            table: [const { MaybeUninit::zeroed() }; 128],
            head: 0,
        }
    }

    pub fn new_process(&mut self, process: Process) {
        self.table[self.head].write(process);
        self.head += 1;
    }
}

pub fn add_process(mut process: Process) -> usize {
    let table = unsafe { &mut PROC_TABLE };
    let head = table.head;
    process.pid = head;
    table.new_process(process);
    head
}

pub fn get_process_at(index: usize) -> &'static Process {
    let table = unsafe { &PROC_TABLE };
    debug_assert!(index <= table.head);

    unsafe { table.table[index].assume_init_ref() }
}

pub fn get_process_at_mut(index: usize) -> &'static mut Process {
    let table = unsafe { &mut PROC_TABLE };
    debug_assert!(index <= table.head);

    unsafe { table.table[index].assume_init_mut() }
}

pub fn iterate_process_table_mut(start_idx: usize) -> impl Iterator<Item = &'static mut Process> {
    let table = unsafe { &mut PROC_TABLE };

    let (left, right) = table.table.split_at_mut(start_idx);

    left.iter_mut()
        .map(|p| unsafe { p.assume_init_mut() })
        .chain(right.iter_mut().map(|p| unsafe { p.assume_init_mut() }))
}
