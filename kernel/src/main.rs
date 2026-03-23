#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use core::{
    arch::{asm, global_asm},
    mem::MaybeUninit,
    panic::PanicInfo,
};

use riscv::registers::{Satp, SatpMode};

use crate::{context::Context, driver::uart::Uart, mm::PhysicalAddress, process::State};
use crate::{helper::*, mm::VirtualAddress};
use crate::{mm::PageTable, process::Process};

const QEMU_TEST: *mut u32 = (KERNEL_DIRECT_MAPPING_BASE + 0x0010_0000) as *mut u32;

pub mod console;
pub mod context;
pub mod driver;
pub mod helper;
pub mod mm;
pub mod plic;
pub mod process;
pub mod trap;
pub(crate) mod userspace;

global_asm!(include_str!("start.s"));

unsafe extern "C" {
    #[allow(unused)]
    fn trap_entry();
}

const UART_PHYSICAL_ADDR: u64 = 0x10000000;

const SYSCALL_WRITE: usize = 1;
const SYSCALL_READ: usize = 2;
const SYSCALL_SLEEP_MS: usize = 3;
const SYSCALL_SHUTDOWN: usize = 4;

const KERNEL_DIRECT_MAPPING_BASE: u64 = 0xffff_ffd6_0000_0000;

pub static EARLY_UART: Uart = Uart::new(UART_PHYSICAL_ADDR as usize);
pub static mut UART: Uart = Uart::new((UART_PHYSICAL_ADDR + KERNEL_DIRECT_MAPPING_BASE) as usize);
pub static mut SCHEDULER_CTX: MaybeUninit<Context> = MaybeUninit::zeroed();

pub static mut KERNEL: Kernel = Kernel {
    current_running_proc: 0,
    n_procs: 0,
    // TODO: temporary queue to store the processes that are blocked by the uart
    uart_wait_queue: [0; 16],
    uart_wait_queue_len: 0,
};

pub static mut PROC_TABLE: [MaybeUninit<Process>; 3] = [const { MaybeUninit::uninit() }; 3];

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
    current_running_proc: usize,
    n_procs: usize,
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
    pub fn create_kernel_process(&mut self, entry: u64) {
        let kernel_stack = mm::alloc().unwrap();
        let kernel_stack_va =
            VirtualAddress::from_raw(kernel_stack.raw() + KERNEL_DIRECT_MAPPING_BASE).unwrap();
        let kernel_sp_va = VirtualAddress::from_raw(kernel_stack_va.raw() + 0x3fa).unwrap();

        let mut context = Context::empty();
        context.ra = entry;
        context.sp = kernel_sp_va.raw();
        unsafe {
            PROC_TABLE[self.n_procs].write(Process {
                pid: self.n_procs,
                kernel_sp: kernel_sp_va.raw(),
                root_table_pa: 0,
                trap_frame: core::ptr::null_mut(),
                context,
                ticks_at_started_running: 0,
                state: State::Ready,
                wake_up_at: 0,
            });
        }
        self.n_procs += 1;
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
                (*process_root_table).map_user_memory(
                    VirtualAddress::from_raw(0x0000_0000_0001_0000 + 0x1000 * i).unwrap(),
                    PhysicalAddress::from_raw_unchecked(entry - 0xffff_ffff_0000_0000 + 0x1000 * i),
                    mm::Perm::Execute,
                    true,
                );
            }
        }

        // 16K stack
        for i in 0..4 {
            let user_stack = mm::alloc().unwrap();

            unsafe {
                (*process_root_table).map_user_memory(
                    VirtualAddress::from_raw(0x0000_0000_3fff_0000 + 0x1000 * i).unwrap(),
                    user_stack,
                    mm::Perm::Write,
                    true,
                )
            };
        }

        let kernel_stack = mm::alloc().unwrap();
        let kernel_stack_va =
            VirtualAddress::from_raw(kernel_stack.raw() + KERNEL_DIRECT_MAPPING_BASE).unwrap();

        unsafe {
            (*process_root_table).map_user_memory(
                kernel_stack_va,
                kernel_stack,
                mm::Perm::Write,
                false,
            )
        };

        mm::kvm_full_map(unsafe { process_root_table.as_mut().unwrap() });

        let kernel_sp_va = VirtualAddress::from_raw(kernel_stack_va.raw() + 0x3fa).unwrap();
        unsafe {
            PROC_TABLE[self.n_procs].write(Process {
                pid: self.n_procs,
                kernel_sp: kernel_sp_va.raw(),
                root_table_pa: process_root_table_pa.raw(),
                trap_frame: core::ptr::null_mut(),
                context: Context::empty(),
                ticks_at_started_running: 0,
                state: State::Ready,
                wake_up_at: 0,
            });
        }

        unsafe {
            // We save `size_of::<Process>()` amount in the stack to write
            let kernel_sp = kernel_sp_va.as_ptr_mut::<Process>().sub(1);

            *kernel_sp = PROC_TABLE[self.n_procs].assume_init_ref().clone();
        }

        self.n_procs += 1;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain(hart_id: u64, dtb_pa: u64) -> ! {
    kdebug(b"hello world from kernel\n");

    let mut buf = [0; 20];
    kdebug("hart id: ");
    kdebug(u64_to_str(hart_id, &mut buf));
    kdebug("dtb pa: ");
    kdebug(u64_to_str_hex(dtb_pa, &mut buf));

    let magic = u32::from_be(unsafe { *(dtb_pa as *const u32) });
    kdebug("magic: ");
    kdebug(u64_to_str_hex(magic as u64, &mut buf));

    enter_supervisor(supervisor_main_no_virtual_memory as *const () as usize);
}

#[inline(always)]
pub fn enter_supervisor(entry: usize) -> ! {
    // `mret` will jump to `mepc` which is `entry`.
    riscv::registers::Mepc::new(entry as u64).write();
    // Enable the supervisor mode so that `mret` starts executing in the S-mode.
    riscv::registers::Mstatus::read()
        .enable_supervisor_mode()
        .write();
    // Delegate all interrupt handlings to S-mode.
    riscv::registers::Mideleg::empty().delegate_all().write();
    riscv::registers::Medeleg::empty().delegate_all().write();

    // Enable access to `rdtime` pseudo-instruction by the S-mode.
    riscv::registers::Mcounteren::empty()
        .enable_access_to_time()
        .write();

    // Enable the `stimecmp` register in S-mode.
    riscv::registers::Menvcfg::empty().enable_stimecmp().write();

    // Enable access to all memory.
    // TODO: Idk if we need to do anything here because we already do memory management in the
    // S-mode. Let's check what Linux does here.
    riscv::registers::Pmpaddr0::new(0x2fffffffffffffff).write();
    riscv::registers::Pmpcfg0::empty()
        .enable_tor()
        .set_readable()
        .set_writable()
        .set_executable()
        .write();

    // Return to address at `mepc`(entry) and start executing in the mode set in `mstatus`(S-mode)
    riscv::mret();
}

/// The entrypoint for when the initial boot phase is done and M-mode switches
/// to S-mode. Only responsibility here is to setup the kernel virtual memory,
/// and immediately switch to the higher base kernel code at [`kernel_higher_half_entry`].
/// Eg. 0x80000000 -> 0xffffffff80000000
#[unsafe(no_mangle)]
pub extern "C" fn supervisor_main_no_virtual_memory() -> ! {
    kdebug(b"hello from the supervisor\n");

    mm::init();

    unsafe {
        asm!(
            "li t0, {kernel_offset}",
            "add t0, t0, {}",
            "jr t0",
            in(reg) kernel_higher_half_entry as *const () as u64,
            kernel_offset = const (mm::KERNEL_IMAGE_START_VA.raw() - mm::KERNEL_IMAGE_START_PA.raw()), 
            options(noreturn, nostack, preserves_flags))
    }
}

/// The actual entry of the kernel. This function assumes that it is running on `S-mode` and the kernel virtual memory
/// is already initialized. It does all the remaining kernel initializations and switches to the first userspace program.
/// It does not return because it explicitly jumps to U-mode with `sret`.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_higher_half_entry() -> ! {
    let kernel_addr = unsafe { &KERNEL as *const Kernel as u64 };
    let mut buf = [0; 20];
    kdebug(b"kernel is loaded at after paging: ");
    kdebug(u64_to_str_hex(kernel_addr, &mut buf));

    plic::plic_init_uart(0);
    unsafe {
        UART.enable_interrupts();
    }

    unsafe {
        KERNEL.create_kernel_process(idle_task as *const () as u64);
        KERNEL.create_process(userspace::shell::shell as *const () as u64);
        KERNEL.create_process(userspace::userspace_sleep_print_loop as *const () as u64);
    };

    unsafe {
        KERNEL.current_running_proc = 1;
    }
    let process = unsafe { PROC_TABLE[1].assume_init_ref() };

    enter_usermode(
        process::PROC_TEXT_VA,
        trap_entry as *const () as u64,
        process::PROC_STACK_VA,
        process.kernel_sp,
        process.root_table_pa,
    );
}

#[inline(never)]
pub fn enter_usermode(
    entry: u64,
    trap_handler: u64,
    user_stack: u64,
    kernel_stack: u64,
    user_root_table_pa: u64,
) -> ! {
    let kernel_satp = Satp::read();

    riscv::write_satp(
        Satp::empty()
            .set_mode(SatpMode::Sv39)
            .set_ppn(user_root_table_pa),
    );

    riscv::registers::Sepc::new(entry).write();

    riscv::registers::Sstatus::read()
        .enable_user_mode()
        .enable_supervisor_interrupts()
        .enable_user_page_access()
        .write();

    riscv::registers::Stvec::new(trap_handler).write();

    unsafe {
        *(kernel_stack as *mut u64) = kernel_satp.raw();
    }

    riscv::registers::Sscratch::new(kernel_stack).write();

    const TIMER_FREQ: u64 = 10_000_000;

    fn ms_to_ticks(ms: u64) -> u64 {
        ms * TIMER_FREQ / 1000
    }

    let time = riscv::registers::Time::read().raw();
    riscv::registers::Stimecmp::new(time + ms_to_ticks(8)).write();
    riscv::registers::Sie::empty()
        .enable_external_interrupts()
        .enable_timer_interrupt()
        .write();

    riscv::sret(user_stack);
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
