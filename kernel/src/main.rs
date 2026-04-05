#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};

use riscv::registers::{Satp, SatpMode};

use crate::{arch::MemoryModel, helper::*};
use crate::{
    arch::PhysicalAddressOf,
    task::{Process, ProcessState},
};
use crate::{
    arch::{
        Architecture, Context, ContextOf, TrapFrame, TrapFrameOf,
        mmu::{PageTable, PhysicalAddress, PteFlags, VirtualAddress},
    },
    driver::uart::Uart,
};

#[cfg(feature = "riscv")]
pub type Arch = arch::Riscv;

const QEMU_TEST: *mut u32 = (KERNEL_DIRECT_MAPPING_BASE + 0x0010_0000) as *mut u32;

pub mod arch;
pub mod console;
pub mod driver;
pub mod helper;
pub mod mm;
pub mod plic;
pub mod syscall;
pub mod task;
pub(crate) mod userspace;

global_asm!(include_str!("start.s"));

unsafe extern "C" {
    #[allow(unused)]
    fn trap_entry();
}

const UART_PHYSICAL_ADDR: u64 = 0x10000000;

const KERNEL_DIRECT_MAPPING_BASE: u64 = 0xffff_ffd6_0000_0000;

pub static EARLY_UART: Uart = Uart::new(UART_PHYSICAL_ADDR as usize);
pub static mut UART: Uart = Uart::new((UART_PHYSICAL_ADDR + KERNEL_DIRECT_MAPPING_BASE) as usize);

pub static mut KERNEL: Kernel = Kernel {
    // TODO: temporary queue to store the processes that are blocked by the uart
    uart_wait_queue: [0; 16],
    uart_wait_queue_len: 0,
};

pub const DEBUG_LEVEL: DebugLevel = {
    if let Some(debug_level) = option_env!("DEBUG_LEVEL") {
        match debug_level.as_bytes()[0] {
            b'0' => DebugLevel::Trace,
            b'1' => DebugLevel::Debug,
            b'2' => DebugLevel::Info,
            _ => DebugLevel::None,
        }
    } else {
        DebugLevel::None
    }
};

#[repr(C)]
pub struct Kernel {
    uart_wait_queue: [usize; 16],
    uart_wait_queue_len: usize,
}

#[derive(PartialEq, PartialOrd)]
pub enum DebugLevel {
    Trace,
    Debug,
    Info,
    None,
}

pub fn ktrace<T: AsRef<[u8]>>(b: T) {
    if DEBUG_LEVEL > DebugLevel::Trace {
        return;
    }
    kprint("[KTRACE] ");
    kprint(b)
}

pub fn kdebug<T: AsRef<[u8]>>(b: T) {
    if DEBUG_LEVEL > DebugLevel::Debug {
        return;
    }
    kprint("[KDEBUG] ");
    kprint(b)
}

pub fn kinfo<T: AsRef<[u8]>>(b: T) {
    if DEBUG_LEVEL > DebugLevel::Info {
        return;
    }
    kprint("[KINFO] ");
    kprint(b)
}

pub fn kprint<T: AsRef<[u8]>>(b: T) {
    let satp = Satp::read();

    let uart_addr = if satp.raw() == 0 {
        UART_PHYSICAL_ADDR
    } else {
        UART_PHYSICAL_ADDR + KERNEL_DIRECT_MAPPING_BASE
    };

    b.as_ref()
        .into_iter()
        .for_each(|b| unsafe { core::ptr::write_volatile(uart_addr as *mut u8, *b) });
}

impl Kernel {
    #[inline(never)]
    pub fn create_kernel_process(&mut self, entry: VirtualAddress) {
        let kernel_stack = mm::alloc().unwrap();
        let kernel_stack_va =
            VirtualAddress::from_raw(kernel_stack.raw() + KERNEL_DIRECT_MAPPING_BASE).unwrap();
        let kernel_sp_va = VirtualAddress::from_raw(kernel_stack_va.raw() + 0x3fa).unwrap();
        let context = ContextOf::<Arch>::initialize(entry, kernel_sp_va);

        task::add_process(Process {
            pid: 0,
            kernel_sp: kernel_sp_va.raw(),
            root_table: PhysicalAddress::ZERO,
            trap_frame: core::ptr::null_mut(),
            context,
            ticks_at_started_running: 0,
            state: ProcessState::Ready,
            wake_up_at: 0,
        });
    }

    #[inline(never)]
    pub fn create_process(&mut self, entry: u64) {
        // we first initiate user's root page table
        let process_root_table_pa = mm::alloc().unwrap();
        let process_root_table_va =
            VirtualAddress::from_raw(process_root_table_pa.raw() + KERNEL_DIRECT_MAPPING_BASE)
                .unwrap();
        let process_root_table = process_root_table_va.as_ptr_mut();
        unsafe { *process_root_table = PageTable::empty() };

        // we don't do heap for now
        // TODO: we temporarily load the user process from the kernel by just mapping it in the userspace

        // Assuming the code is at most 32K
        for i in 0..8 {
            unsafe {
                (*process_root_table).map_vm(
                    VirtualAddress::from_raw(0x0000_0000_0001_0000 + 0x1000 * i).unwrap(),
                    PhysicalAddress::from_raw_unchecked(entry - 0xffff_ffff_0000_0000 + 0x1000 * i),
                    PteFlags::RX | PteFlags::U,
                );
            }
        }

        // 16K stack
        for i in 0..4 {
            let user_stack = mm::alloc().unwrap();

            unsafe {
                (*process_root_table).map_vm(
                    VirtualAddress::from_raw(0x0000_0000_3fff_0000 + 0x1000 * i).unwrap(),
                    user_stack,
                    PteFlags::RW | PteFlags::U,
                )
            };
        }

        let kernel_stack = mm::alloc().unwrap();
        let kernel_stack_va =
            VirtualAddress::from_raw(kernel_stack.raw() + KERNEL_DIRECT_MAPPING_BASE).unwrap();

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
            root_table: process_root_table_pa,
            trap_frame: trap_frame_ptr.as_ptr_mut(),
            context,
            ticks_at_started_running: 0,
            state: ProcessState::Ready,
            wake_up_at: 0,
        });
    }
}

/// The actual entry of the kernel. This function assumes that it is running on `S-mode` and the kernel virtual memory
/// is already initialized. It does all the remaining kernel initializations and switches to the first userspace program.
/// It does not return because it explicitly jumps to U-mode with `sret`.
pub fn kmain() -> ! {
    let kernel_addr = unsafe { &KERNEL as *const Kernel as u64 };
    let mut buf = [0; 20];
    kdebug(b"kernel is loaded at after paging: ");
    kdebug(u64_to_str_hex(kernel_addr, &mut buf));

    plic::plic_init_uart(0);
    unsafe {
        UART.enable_interrupts();
    }

    unsafe {
        KERNEL.create_kernel_process(
            VirtualAddress::from_raw(idle_task as *const () as u64).unwrap(),
        );
        KERNEL.create_process(userspace::shell::shell as *const () as u64);
        KERNEL.create_process(userspace::userspace_sleep_print_loop as *const () as u64);
    };

    let process = task::get_process_at(1);

    Arch::set_root_page_table(process.root_table);

    Arch::set_trap_handler(trap_entry as *const () as u64 as usize);

    Arch::set_kernel_sp(process.kernel_sp as usize);

    let time = Arch::read_current_time();
    Arch::set_timer(time + Arch::nanos_to_ticks(8 * 1_000_000));

    Arch::enable_interrupts();
    Arch::start_usermode(task::PROCESS_TEXT_ADDRESS, task::PROCESS_STACK_ADDRESS);
}

#[unsafe(no_mangle)]
#[inline(never)]
extern "C" fn idle_task() {
    kdebug("idle task running in kernel mode");

    loop {
        riscv::registers::Sstatus::read()
            .enable_supervisor_interrupts()
            .write();
        unsafe {
            asm!("wfi");
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprint("panicked\n");
    match _info.location() {
        Some(loc) => {
            kprint("File: ");
            kprint(loc.file());
            kprint("\nLine: ");
            kprint(u64_to_str(loc.line() as u64, &mut [0; 20]));
            kprint("Column: ");
            kprint(u64_to_str(loc.column() as u64, &mut [0; 20]));
        }
        None => {}
    }
    halt()
}

#[unsafe(no_mangle)]
#[inline(never)]
pub extern "C" fn halt() -> ! {
    unsafe {
        loop {
            core::ptr::write_volatile(QEMU_TEST, 0x5555);
        }
    }
}
