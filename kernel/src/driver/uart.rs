#![allow(unused)]

use crate::mm::KERNEL_DIRECT_MAPPING_BASE;

const UART_IER: usize = 1;
const UART_IIR: usize = 2;
const UART_LCR: usize = 3;
const UART_RHR: usize = 0;
const UART_LSR: usize = 5;

const BUF_SIZE: usize = 1024;

const UART_PHYSICAL_ADDR: usize = 0x10000000;

static mut UART: Uart = Uart::new(UART_PHYSICAL_ADDR + KERNEL_DIRECT_MAPPING_BASE.raw());

/// The uart driver
struct Uart {
    /// Base address of the UART device
    base: usize,

    // TODO: this is basically a ringbuffer, we should use a no_std ringbuf instead of this
    buffer: [u8; BUF_SIZE],
    begin_pos: usize,
    end_pos: usize,
}

pub fn enable_interrupts() {
    let uart = unsafe { &UART };
    unsafe {
        // TODO: check the parity stuff
        core::ptr::write_volatile((uart.base + UART_LCR) as *mut u8, 0b11);
        core::ptr::write_volatile((uart.base + UART_IER) as *mut u8, 0b1);
    }
}

pub fn read_char_into_buf() -> Option<u8> {
    let uart = unsafe { &mut UART };
    let lsr = unsafe { core::ptr::read_volatile((uart.base + UART_LSR) as *const u8) };

    if (lsr & 1) == 0 {
        None
    } else {
        let c = unsafe { core::ptr::read_volatile((uart.base + UART_RHR) as *const u8) };
        uart.buffer[uart.end_pos] = c;
        if uart.end_pos + 1 >= BUF_SIZE {
            uart.end_pos = 0;
        } else {
            uart.end_pos += 1;
        }
        Some(c)
    }
}

/// Tries reading a single character from the device. It's nonblocking so if the
/// device is not ready for read, it returns `None`.
pub fn try_get_char() -> Option<u8> {
    let uart = unsafe { &mut UART };
    if uart.begin_pos == uart.end_pos {
        return None;
    }

    let c = uart.buffer[uart.begin_pos];
    if uart.begin_pos + 1 >= BUF_SIZE {
        uart.begin_pos = 0;
    } else {
        uart.begin_pos += 1;
    }

    Some(c)
}

/// Reads a single character from the device and blocks until a data is read.
pub fn get_char() -> u8 {
    loop {
        if let Some(c) = try_get_char() {
            return c;
        }
    }
}

/// Writes a single character to the device.
pub fn put_char(c: u8) {
    unsafe {
        let uart = unsafe { &UART };
        core::ptr::write_volatile(uart.base as *mut u8, c);
    }
}

impl Uart {
    const fn new(base: usize) -> Self {
        Self {
            base,
            buffer: [0; 1024],
            begin_pos: 0,
            end_pos: 0,
        }
    }
}
