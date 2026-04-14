#![no_std]
#![no_main]

core::arch::global_asm!(include_str!("start.s"));

#[unsafe(no_mangle)]
extern "C" fn bootentry(hartid: usize, dtb_address: usize) {
    let mut buf = [0; 20];
    let _ = u64_to_str(hartid as u64, &mut buf);

    for i in buf {
        console_putchar(i);
    }

    let _ = u64_to_str_hex(dtb_address as u64, &mut buf);

    for i in buf {
        console_putchar(i);
    }
}

#[inline(always)]
fn sbi_call(eid: usize, fid: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let ret;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("a0") arg0 => ret,
            in("a1") arg1,
            in("a2") arg2,
            in("a6") fid,
            in("a7") eid,
        );
    }
    ret
}

pub fn console_putchar(c: u8) {
    sbi_call(0x01, 0, c as usize, 0, 0);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

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
