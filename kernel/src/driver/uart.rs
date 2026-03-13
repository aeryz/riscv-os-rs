const UART_RHR: usize = 0;
const UART_LSR: usize = 5;

/// The uart driver
pub struct Uart {
    /// Base address of the UART device
    base: usize,
}

impl Uart {
    pub const fn new(base: usize) -> Self {
        Self { base }
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
}
