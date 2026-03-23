pub const KB: usize = 1 << 10;
pub const MB: usize = 1 << 20;
pub const GB: usize = 1 << 30;

pub fn u64_to_str(mut n: u64, buf: &mut [u8]) -> &[u8] {
    if buf.is_empty() {
        return b"";
    }

    if n == 0 {
        buf[0] = b'0';
        buf[1] = b'\n';
        return &buf[..2];
    }

    let mut i = 0;

    while n > 0 && i < buf.len() {
        let digit = (n % 10) as u8;
        buf[i] = b'0' + digit;
        n /= 10;
        i += 1;
    }

    buf[..i].reverse();

    buf[i] = b'\n';
    i += 1;

    &buf[..i]
}

pub fn u64_to_str_hex(mut n: u64, buf: &mut [u8]) -> &[u8] {
    if buf.is_empty() {
        return b"";
    }

    if n == 0 {
        buf[0] = b'0';
        buf[1] = b'\n';
        return &buf[..2];
    }

    let mut i = 0;

    while n > 0 && i < buf.len() {
        let digit = (n % 16) as u8;

        buf[i] = match digit {
            0..=9 => b'0' + digit,
            10..=15 => b'a' + (digit - 10),
            _ => unreachable!(),
        };

        n /= 16;
        i += 1;
    }

    buf[..i].reverse();

    buf[i] = b'\n';
    i += 1;

    &buf[..i]
}
