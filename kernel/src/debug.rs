use ksync::SpinLock;

pub const KB: usize = 1 << 10;
pub const MB: usize = 1 << 20;
pub const GB: usize = 1 << 30;

pub const DEBUG_LEVEL: DebugLevel = {
    if let Some(debug_level) = option_env!("DEBUG_LEVEL") {
        match debug_level.as_bytes()[0] {
            b'0' => DebugLevel::Trace,
            b'1' => DebugLevel::Debug,
            b'2' => DebugLevel::Info,
            _ => DebugLevel::None,
        }
    } else {
        DebugLevel::None
    }
};

pub fn usize_to_str(mut n: usize, buf: &mut [u8]) -> &[u8] {
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

pub fn usize_to_str_hex(mut n: usize, buf: &mut [u8]) -> &[u8] {
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

#[derive(PartialEq, PartialOrd)]
pub enum DebugLevel {
    Trace,
    Debug,
    Info,
    None,
}

pub fn ktrace<T: AsRef<[u8]>>(b: T) {
    if DEBUG_LEVEL > DebugLevel::Trace {
        return;
    }
    kprint("[KTRACE] ");
    kprint(b)
}

static CONSOLE_LOCK: SpinLock<()> = SpinLock::new(());

pub fn kdebug<T: AsRef<[u8]>>(b: T) {
    // TODO(aeryz): THIS IS GONNA 100% DEADLOCK UNLESS WE MAKE SURE THE INTERRUPTS ARE DISABLED.
    // maybe a interrupt disabling spinlock where calling `lock` always disables interrupts first?
    let _console_guard = CONSOLE_LOCK.lock();
    if DEBUG_LEVEL > DebugLevel::Debug {
        return;
    }
    kprint("[KDEBUG] ");
    kprint(b);
}

pub fn kinfo<T: AsRef<[u8]>>(b: T) {
    if DEBUG_LEVEL > DebugLevel::Info {
        return;
    }
    kprint("[KINFO] ");
    kprint(b)
}

pub fn kfatal<T: AsRef<[u8]>>(b: T) {
    kprint(b)
}

pub fn kprint<T: AsRef<[u8]>>(b: T) {
    b.as_ref()
        .into_iter()
        .for_each(|b| riscv::sbi::console_putchar(*b));
}
