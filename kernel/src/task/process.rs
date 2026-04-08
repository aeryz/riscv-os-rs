use crate::{
    Arch,
    arch::{
        Architecture, Context, ContextOf, TrapFrame, TrapFrameOf,
        mmu::{PageTable, PhysicalAddress, PteFlags, VirtualAddress},
    },
    mm::{self, ADDRESS_SPACE_EMPTY, AddressSpace, KERNEL_DIRECT_MAPPING_BASE},
    task,
};

pub const PROCESS_TEXT_ADDRESS: VirtualAddress =
    unsafe { VirtualAddress::from_raw_unchecked(0x1_0000) };
pub const PROCESS_STACK_ADDRESS: VirtualAddress =
    unsafe { VirtualAddress::from_raw_unchecked(0x0000_0000_3fff_3fa0) };

#[derive(Clone)]
#[repr(C)]
pub struct Process {
    // TODO: pid is only the index in the proc table right now
    /// Process ID
    pub pid: usize,
    /// Kernel stack pointer
    pub kernel_sp: u64,
    /// Trap frame
    pub trap_frame: *mut TrapFrameOf<Arch>,
    /// Context
    pub context: ContextOf<Arch>,
    /// The tick count at when the process started running
    pub ticks_at_started_running: usize,
    /// The current state of the process
    pub state: ProcessState,
    /// Wake up time in ticks
    pub wake_up_at: usize,
    // TODO(aeryz): We can consider putting this exit code into the relevant state enum
    /// Process exit code
    pub exit_code: i32,

    pub address_space: AddressSpace,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum ProcessState {
    Sleeping,
    Running,
    Ready,
    Blocked,
    /// This task cannot run anymore, but it's address space is not freed also.
    Zombie,
    Exited,
}

/// Creates a kernel process
pub fn create_kernel_process(entry: VirtualAddress) {
    let kernel_stack = mm::alloc().unwrap();
    let kernel_stack_va =
        VirtualAddress::from_raw(kernel_stack.raw() + KERNEL_DIRECT_MAPPING_BASE.raw()).unwrap();
    let kernel_sp_va = VirtualAddress::from_raw(kernel_stack_va.raw() + 0x3fa).unwrap();
    let context = ContextOf::<Arch>::initialize(entry, kernel_sp_va);

    task::add_process(Process {
        pid: 0,
        kernel_sp: kernel_sp_va.raw(),
        trap_frame: core::ptr::null_mut(),
        context,
        ticks_at_started_running: 0,
        state: ProcessState::Ready,
        wake_up_at: 0,
        exit_code: -1,
        address_space: ADDRESS_SPACE_EMPTY,
    });
}

/// Creates a process and adds it to the process table
///
// TODO(aeryz): Note that this is still a temporary implementation because there's no
// file system or ELF support. We just construct the memory mappings for the process
// as if it's being loaded from the filesystem.
pub fn create_process(entry: usize) -> usize {
    // we first initiate user's root page table
    let process_root_table_pa = mm::alloc().unwrap();
    let process_root_table_va =
        VirtualAddress::from_raw(process_root_table_pa.raw() + KERNEL_DIRECT_MAPPING_BASE.raw())
            .unwrap();
    let process_root_table = process_root_table_va.as_ptr_mut();
    unsafe { *process_root_table = PageTable::empty() };

    let mut address_space = ADDRESS_SPACE_EMPTY;
    address_space.root_pt = process_root_table_pa;
    let mut i_region = 0;

    // we don't do heap for now
    // TODO: we temporarily load the user process from the kernel by just mapping it in the userspace

    // Assuming the code is at most 32K
    for i in 0..8 {
        let va = VirtualAddress::from_raw(0x0000_0000_0001_0000 + 0x1000 * i).unwrap();
        unsafe {
            (*process_root_table).map_vm(
                va,
                PhysicalAddress::from_raw_unchecked(
                    entry as u64 - 0xffff_ffff_0000_0000 + 0x1000 * i,
                ),
                PteFlags::RX | PteFlags::U,
            );
        }
        address_space.regions[i_region] = Some(mm::VmRegion {
            start: va,
            end: VirtualAddress::from_raw(va.raw() + 4096).unwrap(),
            process_owned: false,
        });
        i_region += 1;
    }

    // 16K stack
    for i in 0..4 {
        let user_stack = mm::alloc().unwrap();

        let va = VirtualAddress::from_raw(0x0000_0000_3fff_0000 + 0x1000 * i).unwrap();
        unsafe { (*process_root_table).map_vm(va, user_stack, PteFlags::RW | PteFlags::U) };
        address_space.regions[i_region] = Some(mm::VmRegion {
            start: va,
            end: VirtualAddress::from_raw(va.raw() + 4096).unwrap(),
            process_owned: true,
        });
        i_region += 1;
    }

    let kernel_stack = mm::alloc().unwrap();
    let kernel_stack_va =
        VirtualAddress::from_raw(kernel_stack.raw() + KERNEL_DIRECT_MAPPING_BASE.raw()).unwrap();

    address_space.regions[i_region] = Some(mm::VmRegion {
        start: kernel_stack_va,
        end: VirtualAddress::from_raw(kernel_stack_va.raw() + 4096).unwrap(),
        process_owned: true,
    });

    unsafe { (*process_root_table).map_vm(kernel_stack_va, kernel_stack, PteFlags::RW) };

    mm::kvm_full_map(unsafe { process_root_table.as_mut().unwrap() });

    let kernel_sp_va = VirtualAddress::from_raw(kernel_stack_va.raw() + 0x3fa).unwrap();
    let trap_frame_ptr =
        VirtualAddress::from_raw(kernel_sp_va.raw() - size_of::<TrapFrameOf<Arch>>() as u64)
            .unwrap();
    unsafe {
        *(trap_frame_ptr.as_ptr_mut()) = TrapFrameOf::<Arch>::initialize(
            task::PROCESS_TEXT_ADDRESS,
            task::PROCESS_STACK_ADDRESS,
        );
    }

    let context = ContextOf::<Arch>::initialize(
        VirtualAddress::from_raw(Arch::trap_resume_ptr() as u64).unwrap(),
        trap_frame_ptr,
    );

    task::add_process(Process {
        pid: 0,
        kernel_sp: kernel_sp_va.raw(),
        trap_frame: trap_frame_ptr.as_ptr_mut(),
        context,
        ticks_at_started_running: 0,
        state: ProcessState::Ready,
        wake_up_at: 0,
        exit_code: -1,
        address_space,
    })
}

pub fn exit_process(process: &mut Process, exit_code: i32) {
    process.state = task::ProcessState::Zombie;
    process.exit_code = exit_code;

    task::get_process_at_mut(task::TASK_PID_REAPER).state = task::ProcessState::Ready;
}

pub fn reap_process(process: &mut Process) {
    process.state = crate::task::ProcessState::Exited;

    // SAFETY:
    // All the valid processes have root page table
    let root_table = unsafe {
        ((process.address_space.root_pt.raw() + KERNEL_DIRECT_MAPPING_BASE.raw())
            as *const PageTable)
            .as_ref()
            .unwrap()
    };

    process
        .address_space
        .regions
        .iter()
        .filter_map(|r| r.as_ref())
        .filter(|r| r.process_owned)
        .for_each(|r| {
            let mut i = r.start;
            while i.raw() < r.end.raw() {
                crate::kprint("handling: ");
                crate::kprint(crate::u64_to_str_hex(i.raw(), &mut [0; 20]));
                let pa = root_table
                    .translate(i)
                    .expect("All the virtual addresses in an address space must be valid");
                crate::kprint("translated to: ");
                crate::kprint(crate::u64_to_str_hex(pa.raw(), &mut [0; 20]));

                mm::free(pa);

                // SAFETY:
                // Processes are expected to have valid address space
                i = unsafe { VirtualAddress::from_raw_unchecked(i.raw() + 4096) };
            }
        });
}
