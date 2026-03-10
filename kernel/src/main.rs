#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use core::{
    arch::{asm, global_asm}, mem::MaybeUninit, panic::PanicInfo, ptr
};

use riscv::registers::{Satp, SatpMode};

use crate::{allocator::Allocator, memory::physical_address::PhysicalAddress};
use crate::{helper::*, memory::virtual_address::VirtualAddress};
use crate::{
    memory::page_table::{self, PageTable},
    process::Process,
};

const QEMU_TEST: *mut u32 = 0x0010_0000 as *mut u32;

pub mod allocator;
pub mod helper;
pub mod memory;
pub mod process;
pub mod trap;

global_asm!(include_str!("start.s"));

unsafe extern "C" {
    #[allow(unused)]
    fn trap_entry();
}

const UART_PHYSICAL_ADDR: u64 = 0x10000000;

const SYSCALL_WRITE: usize = 1;

const KERNEL_DIRECT_MAPPING_BASE: u64 = 0xffff_ffd6_0000_0000;
const KERNEL_VA_BASE: u64 = 0xffff_ffff_8020_0000;
const KERNEL_PA_BASE: u64 = 0x8020_0000;

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

#[repr(C)]
pub struct Kernel {
    allocator: Allocator<4>,
    root_page_table: *mut PageTable,
    current_running_proc: usize,
    n_procs: usize,
}

pub fn debug<T: AsRef<[u8]>>(b: T) {
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

    debug(b"[kernel] the text page count is: ");
    let mut buf = [0; 20];
    debug(u64_to_str(n_text_pages, &mut buf));

    for _ in 0..n_text_pages {
        root_page_table.create_identity_mapped_page(
            text_start,
            &mut allocator,
            page_table::Perm::Execute,
        );
        text_start = unsafe { PhysicalAddress::from_raw_unchecked(text_start.raw() + 4096) };
    }
    debug(b"[kernel] kvm full mapped \n");

    debug(b"[kernel] right before enabling paging\n");

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
        debug(b"loading the first user process\n");
        // we first initiate user's root page table
        let process_root_table_pa = self.allocator.alloc().unwrap();
        let process_root_table_va =
            VirtualAddress::from_raw(process_root_table_pa.raw() + KERNEL_DIRECT_MAPPING_BASE)
                .unwrap();
        let process_root_table = process_root_table_va.as_ptr_mut();
        unsafe { *process_root_table = PageTable::empty() };

        // we don't do heap for now
        // TODO: we temporarily load the user process from the kernel by just mapping it in the userspace
        unsafe {
            (*process_root_table).map_user_memory(
                VirtualAddress::from_raw(0x0000_0000_0001_0000).unwrap(),
                PhysicalAddress::from_raw_unchecked(entry - 0xffff_ffff_0000_0000),
                &mut self.allocator,
                page_table::Perm::Execute,
                true,
            )
        };
        // TODO: this mapping is also needed since the `entry` might refer to addresses between 0x11000-0x12000
        unsafe {
            (*process_root_table).map_user_memory(
                VirtualAddress::from_raw(0x0000_0000_0001_1000).unwrap(),
                PhysicalAddress::from_raw_unchecked(entry - 0xffff_ffff_0000_0000 + 0x1000),
                &mut self.allocator,
                page_table::Perm::Read,
                true,
            )
        };

        let user_stack = self.allocator.alloc().unwrap();

        unsafe {
            (*process_root_table).map_user_memory(
                VirtualAddress::from_raw(0x0000_0000_3fff_0000).unwrap(),
                user_stack,
                &mut self.allocator,
                page_table::Perm::Write,
                true,
            )
        };

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
            });
        }

        unsafe {
            let mut buf = [0; 20];
            // We save `size_of::<Process>()` amount in the stack to write
            let kernel_sp = kernel_sp_va.as_ptr_mut::<Process>().sub(1);
            debug("current stack pointer is: ");
            debug(u64_to_str_hex(kernel_sp_va.raw(), &mut buf));
            debug("we save the process to: ");
            debug(u64_to_str_hex(kernel_sp as u64, &mut buf));

            *kernel_sp = PROC_TABLE[self.n_procs].assume_init_ref().clone();
        }

        self.n_procs += 1;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    debug(b"hello world from kernel\n");

    enter_supervisor(start as *const () as usize);
}

#[inline(always)]
pub fn enter_supervisor(entry: usize) -> ! {
    riscv::registers::Mepc::new(entry as u64).write();
    riscv::registers::Mstatus::read()
        .enable_supervisor_mode()
        .write();
    riscv::registers::Mideleg::empty().delegate_all().write();
    riscv::registers::Medeleg::empty().delegate_all().write();

    riscv::registers::Pmpaddr0::new(0x2fffffffffffffff).write();
    riscv::registers::Pmpcfg0::empty()
        .enable_tor()
        .set_readable()
        .set_writable()
        .set_executable()
        .write();

    riscv::mret();
}

#[unsafe(no_mangle)]
pub extern "C" fn start() -> ! {
    debug(b"hello from the supervisor\n");

    unsafe {
        asm!(
            "li t1, 32",
            "csrs sie, t1" // Timer interrupt enable flag: STIE
        )
    }

    initialize_kernel();
}

#[unsafe(no_mangle)]
pub extern "C" fn kinit_cont() -> ! {
    let kernel_addr = unsafe { &KERNEL as *const Kernel as u64 };
    let mut buf = [0; 20];
    debug(b"kernel is loaded at after paging: ");
    debug(u64_to_str_hex(kernel_addr, &mut buf));

    unsafe {
        KERNEL.create_process(user_proc_1 as *const () as u64);
    };

    unsafe {        
        KERNEL.create_process(user_proc_2 as *const () as u64);
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
        .disable_supervisor_interrupts()
        .enable_user_page_access()
        .write();

    riscv::registers::Stvec::new(trap_handler).write();

    unsafe {
        *(kernel_stack as *mut u64) = kernel_satp.raw();
    }

    riscv::registers::Sscratch::new(kernel_stack).write();

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
