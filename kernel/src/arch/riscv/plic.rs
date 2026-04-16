// TODO(aeryz): this is a temporary file to only support uart

use crate::mm::KERNEL_DIRECT_MAPPING_BASE;

pub const UART0_IRQ: u32 = 10;

const PLIC: usize = 0x0c00_0000 + KERNEL_DIRECT_MAPPING_BASE.raw();

pub const fn plic_priority(irq: u32) -> *mut u32 {
    (PLIC + (irq as usize) * 4) as *mut u32
}

pub const fn plic_senable(hart: usize) -> *mut u32 {
    (PLIC + 0x2080 + hart * 0x100) as *mut u32
}

pub const fn plic_spriority(hart: usize) -> *mut u32 {
    (PLIC + 0x201000 + hart * 0x2000) as *mut u32
}

pub const fn plic_sclaim(hart: usize) -> *mut u32 {
    (PLIC + 0x201004 + hart * 0x2000) as *mut u32
}

#[inline]
pub fn plic_init_uart(hart: usize) {
    unsafe {
        core::ptr::write_volatile(plic_priority(UART0_IRQ), 1);

        let old = core::ptr::read_volatile(plic_senable(hart));
        core::ptr::write_volatile(plic_senable(hart), old | (1 << UART0_IRQ));

        core::ptr::write_volatile(plic_spriority(hart), 0);
    }
}

#[inline]
pub fn plic_claim(hart: usize) -> u32 {
    unsafe { core::ptr::read_volatile(plic_sclaim(hart)) }
}

#[inline]
pub fn plic_complete(hart: usize, irq: u32) {
    unsafe {
        core::ptr::write_volatile(plic_sclaim(hart), irq);
    }
}
