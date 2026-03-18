use crate::UART;

pub fn print<T: AsRef<[u8]>>(msg: T) {
    msg.as_ref()
        .into_iter()
        .for_each(|b| unsafe { UART.put_char(*b) });
}

pub fn println<T: AsRef<[u8]>>(msg: T) {
    print(msg);
    unsafe {
        UART.put_char(b'\n');
    }
}

pub fn getchar() -> u8 {
    unsafe { UART.get_char() }
}
