use crate::UART;

pub fn print<T: AsRef<[u8]>>(msg: T) {
    msg.as_ref().into_iter().for_each(|b| UART.put_char(*b));
}

pub fn println<T: AsRef<[u8]>>(msg: T) {
    print(msg);
    UART.put_char(b'\n');
}

pub fn getchar() -> u8 {
    UART.get_char()
}

pub fn readline(buf: &mut [u8]) -> usize {
    let mut pos = 0;
    loop {
        let c = UART.get_char();
        if c == b'\n' || c == b'\r' {
            return pos;
        }
        buf[pos] = c;
        pos += 1;

        if pos >= buf.len() {
            return pos;
        }
    }
}
