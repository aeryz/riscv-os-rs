use crate::UART;

pub fn println<T: AsRef<[u8]>>(msg: T) {
    msg.as_ref().into_iter().for_each(|b| UART.put_char(*b));
    UART.put_char(b'\n');
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
// let mut pos = 0;
// for _ in 0..10 {
//     let c = uart_getchar_blocking();
//     match c {
//         127 | 8 => {
//             // backspace
//             if pos > 0 {
//                 pos -= 1;

//                 uart_putchar(b'\x08'); // move left
//                 uart_putchar(b' ');
//                 uart_putchar(b'\x08'); // move left again
//             }
//         }

//         // b'\r' | b'\n' => {
//         //     uart_putchar(b'\n');
//         //     buf[pos] = b'\n';
//         //     pos += 1;
//         //     break;
//         // }
//         _ => {
//             if pos < buf.len() {
//                 buf[pos] = c;
//                 pos += 1;
//                 uart_putchar(c); // echo
//             }
//         }
//     }
// }
