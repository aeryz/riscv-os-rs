#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use core::{
    arch::{asm, global_asm},
    mem::MaybeUninit,
    panic::PanicInfo,
    ptr,
};

use riscv::registers::{Satp, SatpMode};

use crate::{allocator::Allocator, context::Context, driver::uart::Uart, memory::physical_address::PhysicalAddress, process::State};
use crate::{helper::*, memory::virtual_address::VirtualAddress};
use crate::{
    memory::page_table::{self, PageTable},
    process::Process,
};

const QEMU_TEST: *mut u32 = 0x0010_0000 as *mut u32;

pub mod allocator;
pub mod console;
pub mod context;
pub mod driver;
pub mod helper;
pub mod memory;
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

const KERNEL_DIRECT_MAPPING_BASE: u64 = 0xffff_ffd6_0000_0000;
const KERNEL_VA_BASE: u64 = 0xffff_ffff_8020_0000;
const KERNEL_PA_BASE: u64 = 0x8020_0000;

pub static EARLY_UART: Uart = Uart::new(UART_PHYSICAL_ADDR as usize);
pub static mut UART: Uart = Uart::new((UART_PHYSICAL_ADDR + KERNEL_DIRECT_MAPPING_BASE) as usize);
pub static mut SCHEDULER_CTX: MaybeUninit<Context> = MaybeUninit::zeroed();

unsafe extern "C" {
    static __text_start: u8;
    static __text_end: u8;
    static __kernel_end: u8;
}

pub static mut KERNEL: Kernel = Kernel {
    allocator: Allocator::new(),
    root_page_table: ptr::null_mut(),
    current_running_proc: 0,
    n_procs: 0,
};

pub static mut PROC_TABLE: [MaybeUninit<Process>; 2] = [const { MaybeUninit::uninit() }; 2];

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
    allocator: Allocator<4>,
    root_page_table: *mut PageTable,
    current_running_proc: usize,
    n_procs: usize,
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

#[inline(never)]
pub fn initialize_kernel() -> ! {
    let memory_start =
        unsafe { PhysicalAddress::from_raw_unchecked(&__kernel_end as *const u8 as u64) };
    let mut allocator = Allocator::new();
    allocator.set_start_addr(memory_start);

    let root_page_table: &mut PageTable = unsafe { &mut *allocator.alloc().unwrap().as_ptr_mut() };

    *root_page_table = PageTable::empty();
    root_page_table.kvm_full_map();

    let text_end = unsafe { &__text_end as *const u8 as u64 };
    let mut text_start =
        unsafe { PhysicalAddress::from_raw_unchecked(&__text_start as *const u8 as u64) };

    let n_text_pages = (text_end - text_start.raw()) / 4096 + 1;

    kdebug(b"the text page count is: ");
    let mut buf = [0; 20];
    kdebug(u64_to_str(n_text_pages, &mut buf));

    for _ in 0..n_text_pages {
        root_page_table.create_identity_mapped_page(
            text_start,
            &mut allocator,
            page_table::Perm::Execute,
        );
        text_start = unsafe { PhysicalAddress::from_raw_unchecked(text_start.raw() + 4096) };
    }
    kdebug(b"kvm full mapped \n");

    unsafe {
        KERNEL.allocator = allocator;
        KERNEL.root_page_table = root_page_table as *mut PageTable;
    }

    riscv::write_satp(
        Satp::empty()
            .set_mode(SatpMode::Sv39)
            .set_ppn(root_page_table as *mut PageTable as u64),
    );

    unsafe {
        asm!(
        "li t0, {kernel_offset}",
        "add sp, sp, t0",
        "add t0, t0, {}",
        "jr t0",
        in(reg) kinit_cont as *const () as u64,
        kernel_offset = const (KERNEL_VA_BASE - KERNEL_PA_BASE), 
        options(noreturn, nostack, preserves_flags))
    }
}

impl Kernel {
    #[inline(never)]
    pub fn create_process(&mut self, entry: u64) {
        // we first initiate user's root page table
        let process_root_table_pa = self.allocator.alloc().unwrap();
        let process_root_table_va =
            VirtualAddress::from_raw(process_root_table_pa.raw() + KERNEL_DIRECT_MAPPING_BASE)
                .unwrap();
        let process_root_table = process_root_table_va.as_ptr_mut();
        unsafe { *process_root_table = PageTable::empty() };

        // we don't do heap for now
        // TODO: we temporarily load the user process from the kernel by just mapping it in the userspace

        // Assuming the code is at most 16K
        for i in 0..4 {
            unsafe {
                (*process_root_table).map_user_memory(
                    VirtualAddress::from_raw(0x0000_0000_0001_0000 + 0x1000 * i).unwrap(),
                    PhysicalAddress::from_raw_unchecked(entry - 0xffff_ffff_0000_0000 + 0x1000 * i),
                    &mut self.allocator,
                    page_table::Perm::Execute,
                    true,
                );
            }
        }
            
        // 16K stack
        for i in 0..4 {
            let user_stack = self.allocator.alloc().unwrap();

            unsafe {
                (*process_root_table).map_user_memory(
                    VirtualAddress::from_raw(0x0000_0000_3fff_0000 + 0x1000 * i).unwrap(),
                    user_stack,
                    &mut self.allocator,
                    page_table::Perm::Write,
                    true,
                )
            };
        }

        let kernel_stack = self.allocator.alloc().unwrap();
        let kernel_stack_va =
            VirtualAddress::from_raw(kernel_stack.raw() + KERNEL_DIRECT_MAPPING_BASE).unwrap();

        unsafe {
            (*process_root_table).map_user_memory(
                kernel_stack_va,
                kernel_stack,
                &mut self.allocator,
                page_table::Perm::Write,
                false,
            )
        };

        unsafe {
            (*process_root_table).kvm_full_map();
        }

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
                wake_up_at: 0
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

    enter_supervisor(start as *const () as usize);
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
    riscv::registers::Mcounteren::empty().enable_access_to_time().write();

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

#[unsafe(no_mangle)]
pub extern "C" fn start() -> ! {
    kdebug(b"hello from the supervisor\n");

    initialize_kernel();
}

#[unsafe(no_mangle)]
pub extern "C" fn kinit_cont() -> ! {
    let kernel_addr = unsafe { &KERNEL as *const Kernel as u64 };
    let mut buf = [0; 20];
    kdebug(b"kernel is loaded at after paging: ");
    kdebug(u64_to_str_hex(kernel_addr, &mut buf));

    plic::plic_init_uart(0);
    unsafe { UART.enable_interrupts(); }

    // unsafe {
    //     KERNEL.create_process(userspace::shell::shell as *const () as u64);
    // };

    unsafe {
        KERNEL.create_process(userspace::shell::shell as *const () as u64);
        KERNEL.create_process(userspace::userspace_2 as *const () as u64);
    };

    let process = unsafe { PROC_TABLE[0].assume_init_ref() };

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
    riscv::registers::Sie::empty().enable_external_interrupts().enable_timer_interrupt().write();

    riscv::sret(user_stack);
}

#[unsafe(no_mangle)]
pub extern "C" fn user_proc_1() {
    unsafe { asm!(".align 12") };
    loop {
        let message = b"[1] this the userspace program\n";
        let message_ptr = message as *const u8;
        let message_len = message.len();

        let ret: isize;

        unsafe {
            asm!(
                "li a0, 1",
                "ecall",
                in("a7") SYSCALL_WRITE,
                in("a1") message_ptr,
                in("a2") message_len,
                lateout("a0") ret,
                options(nostack),
            )
        }

        if ret != -1 {
            let message = b"[1] written to the kernel, cool\n";
            let message_ptr = message as *const u8;
            let message_len = message.len();

            unsafe {
                asm!(
                    "li a0, 1",
                    "ecall",
                    in("a7") SYSCALL_WRITE,
                    in("a1") message_ptr,
                    in("a2") message_len,
                    options(nostack),
                )
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn user_proc_2() -> ! {
    unsafe { asm!(".align 12") };

    loop {
        let message = b"[2] this the userspace program\n";
        let message_ptr = message as *const u8;
        let message_len = message.len();

        let ret: isize;

        unsafe {
            asm!(
                "li a0, 1",
                "ecall",
                in("a7") SYSCALL_WRITE,
                in("a1") message_ptr,
                in("a2") message_len,
                lateout("a0") ret,
                options(nostack),
            )
        }

        if ret != -1 {
            let message = b"[2] written to the kernel, cool\n";
            let message_ptr = message as *const u8;
            let message_len = message.len();

            unsafe {
                asm!(
                    "li a0, 1",
                    "ecall",
                    in("a7") SYSCALL_WRITE,
                    in("a1") message_ptr,
                    in("a2") message_len,
                    options(nostack),
                )
            }
        }
    }
}
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

pub fn halt() {
    unsafe {
        core::ptr::write_volatile(QEMU_TEST, 0x5555);
    }
}
