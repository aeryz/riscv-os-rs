const UART_IER: usize = 1;
const UART_IIR: usize = 2;
const UART_LCR: usize = 3;
const UART_RHR: usize = 0;
const UART_LSR: usize = 5;

/// The uart driver
pub struct Uart {
    /// Base address of the UART device
    base: usize,

    // TODO: this is basically a ringbuffer, we should use a no_std ringbuf instead of this
    buffer: [u8; 1024],
    begin_pos: usize,
    end_pos: usize,
}

impl Uart {
    pub const fn new(base: usize) -> Self {
        Self {
            base,
            buffer: [0; 1024],
            begin_pos: 0,
            end_pos: 0,
        }
    }

    pub fn enable_interrupts(&self) {
        unsafe {
            // TODO: check the parity stuff
            core::ptr::write_volatile((self.base + UART_LCR) as *mut u8, 0b11);
            core::ptr::write_volatile((self.base + UART_IER) as *mut u8, 0b1);
        }
    }

    /// Tries reading a single character from the device. It's nonblocking so if the
    /// device is not ready for read, it returns `None`.
    pub fn try_get_char(&self) -> Option<u8> {
        let lsr = unsafe { core::ptr::read_volatile((self.base + UART_LSR) as *const u8) };

        if (lsr & 1) == 0 {
            None
        } else {
            let c = unsafe { core::ptr::read_volatile((self.base + UART_RHR) as *const u8) };
            Some(c)
        }
    }

    /// Reads a single character from the device and blocks until a data is read.
    pub fn get_char(&self) -> u8 {
        loop {
            if let Some(c) = self.try_get_char() {
                return c;
            }
        }
    }

    /// Writes a single character to the device.
    pub fn put_char(&self, c: u8) {
        unsafe {
            core::ptr::write_volatile(self.base as *mut u8, c);
        }
    }

    pub fn read_iir(&self) -> u8 {
        unsafe { core::ptr::read_volatile((self.base + UART_IIR) as *const u8) }
    }
}
