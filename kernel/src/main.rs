#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
    ptr,
};

use crate::memory::page_table::{self, PageTable};
use crate::{allocator::Allocator, memory::physical_address::PhysicalAddress};
use crate::{helper::*, memory::virtual_address::VirtualAddress};

pub mod allocator;
pub mod helper;
pub mod memory;
pub mod trap;

global_asm!(include_str!("start.s"));

unsafe extern "C" {
    #[allow(unused)]
    fn trap_entry();
}

const UART_PHYSICAL_ADDR: u64 = 0x10000000;

const XSTATUS_XPP_SHIFT: usize = 11;
const XSTATUS_XPP_S: usize = 0b01 << XSTATUS_XPP_SHIFT;
const XSTATUS_MPP_X: usize = 0b11 << XSTATUS_XPP_SHIFT;
const XSTATUS_SIE: usize = 0b1 << 1;
const XSTATUS_SUM: usize = 0b1 << 18;
const PMP_0_CFG: usize = 0b00001111;

const SYSCALL_WRITE: usize = 1;
const SATP_MODE_SV39: u64 = 8;

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
};

#[repr(C)]
pub struct Kernel {
    allocator: Allocator<4>,
    root_page_table: *mut PageTable,
}

pub fn debug<T: AsRef<[u8]>>(b: T) {
    let satp: u64;
    unsafe {
        asm!(
            "csrr {0}, satp",
            out(reg) satp,
        )
    };

    let uart_addr = if satp == 0 {
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
    // root_pa must be 4KiB-aligned
    let ppn = (root_page_table as *mut PageTable as u64) >> 12;
    let satp = (SATP_MODE_SV39 << 60) | ppn;

    unsafe {
        KERNEL.allocator = allocator;
        KERNEL.root_page_table = root_page_table as *mut PageTable;
    }

    unsafe {
        asm!(
        "csrw satp, {}",
        "sfence.vma x0, x0",
        "li t0, {kernel_offset}",
        "add sp, sp, t0",
        "add t0, t0, {}",
        "jr t0",
        in(reg) satp,
        in(reg) kinit_cont as *const () as u64,
        kernel_offset = const (KERNEL_VA_BASE - KERNEL_PA_BASE), 
        options(noreturn, nostack, preserves_flags))
    }
}

impl Kernel {
    #[inline(never)]
    pub fn load_first_process(&mut self) {
        let mut buf = [0; 20];
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
                PhysicalAddress::from_raw_unchecked(
                    user_proc_1 as *const () as u64 - 0xffff_ffff_0000_0000,
                ),
                &mut self.allocator,
                page_table::Perm::Execute,
                true,
            )
        };
        // TODO: this mapping is also needed since the `user_proc_1` refers to addresses between 0x11000-0x12000
        unsafe {
            (*process_root_table).map_user_memory(
                VirtualAddress::from_raw(0x0000_0000_0001_1000).unwrap(),
                PhysicalAddress::from_raw_unchecked(
                    user_proc_1 as *const () as u64 - 0xffff_ffff_0000_0000 + 0x1000,
                ),
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

        unsafe {
            (*process_root_table).map_user_memory(
                VirtualAddress::from_raw(kernel_stack.raw() + KERNEL_DIRECT_MAPPING_BASE).unwrap(),
                kernel_stack,
                &mut self.allocator,
                page_table::Perm::Write,
                false,
            )
        };

        unsafe {
            (*process_root_table).kvm_full_map();
        }

        debug(b"right after mapping the user memory");

        debug(b"the current process root table: ");
        debug(u64_to_str_hex(process_root_table_pa.raw() as u64, &mut buf));

        enter_usermode(
            0x0000_0000_0001_0000,
            trap_entry as *const () as usize,
            0x0000_0000_3fff_0fa0,
            (kernel_stack.raw() + KERNEL_DIRECT_MAPPING_BASE + 0xfa0) as usize,
            process_root_table_pa.raw() as usize,
        );
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    debug(b"hello world from kernel\n");

    enter_supervisor(start as *const () as usize);

    loop {
        core::hint::spin_loop();
    }
}

#[inline(always)]
pub fn mret() {
    unsafe {
        asm!("mret", options(nomem, nostack, preserves_flags));
    }
}

#[inline(always)]
pub fn enter_supervisor(entry: usize) {
    unsafe {
        asm!(
            "csrw mepc, {entry}",

            "csrr t0, mstatus",
            "li t1, {xpp_s}",
            // unset 12th bit for setting the MPP to 01(S mode)
            "or t0, t0, t1",
            "slli t1, t1, 1",
            "not t1, t1",
            "and t0, t0, t1",
            "csrw mstatus, t0",

            // TODO: delegating everything to supervisor right now for ease of use.
            // Need to investigate further to see if we want to handle some traps
            // in the M-level.
            // Delegate all interrupts and traps to the supervisor
            "li t0, -1",
            "csrw medeleg, t0",
            "csrw mideleg, t0",

            // Allow the supervisor to read/write/execute anywhere between 0-0x2fffff..
            "li t0, 0x2fffffffffffffff",
            "csrw pmpaddr0, t0",
            "csrw pmpcfg0, {pmp_cfg}",

            "mret",

            entry = in(reg) entry,
            xpp_s = const XSTATUS_XPP_S,
            pmp_cfg = const PMP_0_CFG,
            options(noreturn)
        )
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn start() -> ! {
    debug(b"hello from the supervisor\n");

    initialize_kernel();
}

#[unsafe(no_mangle)]
pub extern "C" fn kinit_cont() -> ! {
    let kernel_addr = unsafe { &KERNEL as *const Kernel as u64 };
    let mut buf = [0; 20];
    debug(b"kernel is loaded at after paging: ");
    debug(u64_to_str_hex(kernel_addr, &mut buf));

    unsafe { KERNEL.load_first_process() };

    debug(b"damn executed the first user process\n");

    loop {
        core::hint::spin_loop();
    }
}

#[inline(never)]
pub fn enter_usermode(
    entry: usize,
    trap_handler: usize,
    user_stack: usize,
    kernel_stack: usize,
    user_satp: usize,
) {
    let ppn = (user_satp as u64) >> 12;
    let user_satp = (SATP_MODE_SV39 << 60) | ppn;

    unsafe {
        asm!(
            // save the kernel satp for later
            "csrr t2, satp",
            // first switch to the user satp
            "csrw satp, {user_satp}",
            "sfence.vma x0, x0",

            "csrw sepc, {entry}",

            "csrr t0, sstatus",
            // set spp to usermode (00)
            "li t1, {xpp_m}",
            "not t1, t1",
            "and t0, t0, t1",

            // enable trap handler
            "ori t0, t0, {sie}",
            // s-mode can access user accessible pages
            "or t0, t0, {sum}",

            "csrw sstatus, t0",

            // setup the trap handler base address
            "csrw stvec, {trap_handler}",

            // setup kernel stack
            "mv t0, {kernel_sp}",
            "sd t2, 0(t0)",
            "csrw sscratch, t0",

            // TODO: enable scounteren

            "mv sp, {sp}",
            "sret",

            entry = in(reg) entry,
            trap_handler = in(reg) trap_handler,
            sp = in(reg) user_stack,
            user_satp = in(reg) user_satp,
            kernel_sp = in(reg) kernel_stack,
            xpp_m = const XSTATUS_MPP_X,
            sie = const XSTATUS_SIE,
            sum = in(reg) XSTATUS_SUM,

            options(noreturn)
        )
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn user_proc_1() {
    unsafe { asm!(".align 12") };
    let message = b"hello from the userspace\n";
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
        let message = b"written to the kernel, cool\n";
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

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
