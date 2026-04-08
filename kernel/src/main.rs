#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};

use crate::task::ProcessState;
use crate::{arch::MemoryModel, mm::KERNEL_DIRECT_MAPPING_BASE};
use crate::{
    arch::{Architecture, mmu::VirtualAddress},
    driver::uart::Uart,
};

#[cfg(feature = "riscv")]
pub type Arch = arch::Riscv;

const QEMU_TEST: *mut u32 = (KERNEL_DIRECT_MAPPING_BASE.raw() + 0x0010_0000) as *mut u32;

pub mod arch;
pub mod console;
pub mod driver;
pub mod helper;
pub mod mm;
pub mod plic;
pub mod syscall;
pub mod task;
pub(crate) mod userspace;

pub use helper::*;

global_asm!(include_str!("start.s"));

unsafe extern "C" {
    #[allow(unused)]
    fn trap_entry();
}

const UART_PHYSICAL_ADDR: u64 = 0x10000000;

pub static EARLY_UART: Uart = Uart::new(UART_PHYSICAL_ADDR as usize);
pub static mut UART: Uart =
    Uart::new((UART_PHYSICAL_ADDR + KERNEL_DIRECT_MAPPING_BASE.raw()) as usize);

// TODO(aeryz): this is ugly, when we are done with splitting into subsystems, we won't
// have any kernel struct
pub static mut KERNEL: Kernel = Kernel {
    // TODO: temporary queue to store the processes that are blocked by the uart
    uart_wait_queue: [0; 16],
    uart_wait_queue_len: 0,
};

#[repr(C)]
pub struct Kernel {
    uart_wait_queue: [usize; 16],
    uart_wait_queue_len: usize,
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

    task::create_kernel_process(VirtualAddress::from_raw(idle_task as *const () as u64).unwrap());
    task::create_kernel_process(VirtualAddress::from_raw(reaper_task as *const () as u64).unwrap());
    task::get_process_at_mut(task::TASK_PID_REAPER).state = ProcessState::Blocked;

    let init_proc_pid = task::create_process(userspace::shell::shell as *const () as usize);
    let _ = task::create_process(userspace::userspace_sleep_print_loop as *const () as usize);

    let process = task::get_process_at(init_proc_pid);

    task::init_scheduler(process.pid);

    Arch::set_root_page_table(process.address_space.root_pt);

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

#[unsafe(no_mangle)]
#[inline(never)]
extern "C" fn reaper_task() {
    loop {
        kprint("got here brutha\n");
        task::iterate_process_table_mut(0)
            .filter(|p| p.state == task::ProcessState::Zombie)
            .for_each(|p| task::reap_process(p));

        task::schedule(false);
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
