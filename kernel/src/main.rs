#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use core::{
    arch::{asm, global_asm},
    panic::{PanicInfo},
    ptr,
};

use crate::{helper::*, memory::virtual_address::VirtualAddress};
use crate::memory::page_table::{self, PageTable};
use crate::{allocator::Allocator, memory::physical_address::PhysicalAddress};

pub mod allocator;
pub mod helper;
pub mod memory;
pub mod trap;

global_asm!(include_str!("start.s"));

unsafe extern "C" {
    #[allow(unused)]
    fn trap_entry();
}

const UART_ADDR: *mut u8 = 0x10000000 as *mut u8;

const XSTATUS_XPP_SHIFT: usize = 11;
const XSTATUS_XPP_S: usize = 0b01 << XSTATUS_XPP_SHIFT;
const XSTATUS_MPP_X: usize = 0b11 << XSTATUS_XPP_SHIFT;
const XSTATUS_SIE: usize = 0b1 << 1;
const XSTATUS_SUM: usize = 0b1 << 18;
const PMP_0_CFG: usize = 0b00001111;

const SYSCALL_WRITE: usize = 1;
const SATP_MODE_SV39: u64 = 8;

const KERNEL_DIRECT_MAPPING_BASE: u64 = 0xffff_ffc0_0000_0000;
const KERNEL_VA_BASE: u64 = 0xffff_ffff_8020_0000;
const KERNEL_PA_BASE: u64 = 0x8020_0000;

unsafe extern "C" {
    static __text_start: u8;
    static __text_end: u8;
    // static __rodata_start: u8;
    // static __bss_start: u8;
    static __kernel_end: u8;
    static __stack_bottom: u8;
    static __stack_top: u8;
    // static __kernel_stack_bottom: u8;
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

pub fn debug(b: &[u8]) {
    b.into_iter()
        .for_each(|b| unsafe { core::ptr::write_volatile(UART_ADDR, *b) });
}

impl Kernel {
    pub fn initialize(&mut self) {
        let memory_start =
            unsafe { PhysicalAddress::from_raw_unchecked(&__kernel_end as *const u8 as u64) };
        self.allocator.set_start_addr(memory_start);

        self.initialize_page_tables();
    }

    pub fn initialize_page_tables(&mut self) {
        let root_page_table: &mut PageTable = unsafe { &mut *self.allocator.alloc().unwrap().as_ptr_mut() };

        *root_page_table = PageTable::empty();
        root_page_table.kvm_full_map();

        let text_end = unsafe {&__text_end as *const u8 as u64 };
        let mut text_start = unsafe {
            PhysicalAddress::from_raw_unchecked(&__text_start as *const u8 as u64) };

        let n_text_pages = (text_end - text_start.raw()) / 4096 + 1;

        debug(b"[kernel] the text page count is: ".as_slice());
        let mut buf = [0; 20];
        debug(u64_to_str(n_text_pages, &mut buf));

        for _ in 0..n_text_pages {
            root_page_table.create_identity_mapped_page(
                text_start,
                &mut self.allocator,
                page_table::Perm::Execute,
                false,
            );
            text_start = unsafe { PhysicalAddress::from_raw_unchecked(text_start.raw() + 4096) };
        }
        debug(b"[kernel] kvm full mapped \n".as_slice());

        root_page_table.create_identity_mapped_page(
            unsafe { PhysicalAddress::from_raw_unchecked(UART_ADDR as u64) },
            &mut self.allocator,
            page_table::Perm::Write,
            false,
        );

        self.root_page_table = root_page_table;

        debug(b"[kernel] right before enabling paging\n".as_slice());
        self.initiate_paging();
        debug(b"[kernel] right after enabling paging\n".as_slice());
    }

    #[inline(never)]
    pub fn initiate_paging(&mut self) {
        // root_pa must be 4KiB-aligned
        let ppn = (self.root_page_table as u64) >> 12;
        let satp = (SATP_MODE_SV39 << 60) | ppn;

        unsafe {
            asm!(
            "csrw satp, {}",
            "auipc t1, 0",
            "li t0, {kernel_offset}",
            "add t0, t0, t1",
            "jr  t0",
            // flush the tlb
            "sfence.vma x0, x0",
            "li t0, {kernel_offset}",
            "addi t0, t0, -0xe",
            "add sp, sp, t0",
            "add ra, ra, t0",
            in(reg) satp,
            // TODO: adding 0xc here is a nasty hack because above, we load the pc, then we execute a few instructions
            // and only then we jump. 0xe moves the pointer to `sfence.vma`. But manually computing it like this is nasty
            // and error prone. We should change it.
            kernel_offset = const (KERNEL_VA_BASE - KERNEL_PA_BASE + 0xe), 
            options(nostack, preserves_flags))
        }
    }

    #[inline(never)]
    pub fn load_first_process(&mut self) {
        let mut buf = [0; 20];
        debug(b"loading the first user process\n".as_slice());
        // we first initiate user's root page table
        let process_root_table_pa = self.allocator.alloc().unwrap();
        let process_root_table_va = VirtualAddress::from_raw(process_root_table_pa.raw() + KERNEL_DIRECT_MAPPING_BASE).unwrap();
        let process_root_table = process_root_table_va.as_ptr_mut();
        unsafe { *process_root_table = PageTable::empty() };

        // we don't do heap for now
        let code_section_at_kernel = user_proc_1 as *const u8;
        debug(b"[ + ] user proc: ".as_slice());
        debug(u64_to_str_hex(code_section_at_kernel as u64, &mut buf));

        let code_section_pa = self.allocator.alloc().unwrap();
        let code_section_va = VirtualAddress::from_raw(code_section_pa.raw() + KERNEL_DIRECT_MAPPING_BASE).unwrap();
        debug(b"[...] copying the user code\n".as_slice());
        (0..46).for_each(|i| {
            unsafe {
                *((code_section_va.raw() + i) as *mut u8) = *(code_section_at_kernel.add(i as usize));
            }
        });
        debug(b"[ + ] copied the user code\n".as_slice());

        unsafe {
            (*process_root_table).map_user_memory(
                VirtualAddress::from_raw(0x0000_0000_0001_0000).unwrap(),
                code_section_pa,
                &mut self.allocator,
                page_table::Perm::Execute,
                true,
            )
        };
        debug(b"[ + ] after\n".as_slice());

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
                VirtualAddress::from_raw(0x0000_0000_3fff_2000).unwrap(),
                kernel_stack,
                &mut self.allocator,
                page_table::Perm::Write,
                false,
            )
        };

        let text_end = unsafe {&__text_end as *const u8 as u64 };
        let mut text_start = unsafe { VirtualAddress::from_raw(&__text_start as *const u8 as u64).unwrap() };

        let n_text_pages = (text_end - text_start.raw()) / 4096 + 1;

        for _ in 0..n_text_pages {
            debug(b"mapping: ".as_slice());
            debug(u64_to_str_hex(text_start.raw(), &mut buf));
            debug(b"\tinto -> ".as_slice());
            debug(u64_to_str_hex(PhysicalAddress::from_raw(text_start.raw() - 0xffff_ffff_0000_0000).unwrap().raw(), &mut buf));
            unsafe {
                (*process_root_table).map_user_memory(
                    text_start,
                    PhysicalAddress::from_raw(text_start.raw() - 0xffff_ffff_0000_0000).unwrap(),
                    &mut self.allocator,
                    page_table::Perm::Execute,
                    false,
                );
            }
            text_start = VirtualAddress::from_raw(text_start.raw() + 4096).unwrap();
        }

        let trap_entry =
            unsafe { PhysicalAddress::from_raw_unchecked(trap_entry as *const () as u64 - 0xffff_ffff_0000_0000) };
        debug(b"trampoline: ".as_slice());
        debug(u64_to_str_hex(trap_entry.raw(), &mut buf));

        unsafe {
            (*process_root_table).map_user_memory(
                VirtualAddress::from_raw(0x0000_0000_3fff_2000).unwrap(),
                trap_entry,
                &mut self.allocator,
                page_table::Perm::Execute,
                false,
            )
        }

        debug(b"right after mapping the user memory".as_slice());

        debug(b"the current process root table: ".as_slice());
        debug(u64_to_str_hex(process_root_table_pa.raw() as u64, &mut buf));

        enter_usermode(
            0x0000_0000_0001_0000,
            trap_entry.raw() as usize,
            0x0000_0000_3fff_1000,
            0x0000_0000_3fff_3000,
            process_root_table_pa.raw() as usize,
            self.root_page_table as usize,
        );
    }

}

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    b"hello world from kernel\n"
        .into_iter()
        .for_each(|b| unsafe { core::ptr::write_volatile(UART_ADDR, *b) });

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
    debug(b"hello from the supervisor\n".as_slice());

    unsafe { KERNEL.initialize() };

    debug(b"calling load\n".as_slice());

    unsafe { KERNEL.load_first_process() };

    loop {
        core::hint::spin_loop();
    }
}

#[inline(never)]
pub fn enter_usermode(entry: usize, trap_handler: usize, user_stack: usize, kernel_stack: usize, user_satp: usize, kernel_satp: usize) {
    let ppn = (user_satp as u64) >> 12;
    let user_satp = (SATP_MODE_SV39 << 60) | ppn;

    unsafe {
        asm!(
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
            "csrw sscratch, t0",

            "addi t0, t0, 8",
            "mv t1, {kernel_satp}",
            "sd t1, 0(sp)",

            // TODO: enable scounteren

            "mv sp, {sp}",

            "csrw satp, {user_satp}",
            "sfence.vma x0, x0",
            "sret",

            entry = in(reg) entry,
            trap_handler = in(reg) trap_handler,
            sp = in(reg) user_stack,
            user_satp = in(reg) user_satp,
            kernel_sp = in(reg) kernel_stack,
            kernel_satp = in(reg) kernel_satp,
            xpp_m = const XSTATUS_MPP_X,
            sie = const XSTATUS_SIE,
            sum = in(reg) XSTATUS_SUM,

            options(noreturn)
        )
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn user_proc_1() -> ! {
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

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
