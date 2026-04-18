use core::fmt::{self, Write};
use log::{Metadata, Record};

use crate::printk;

const DEBUG_LEVEL: Option<&str> = option_env!("RUST_LOG");
static LOGGER: SerialLogger = SerialLogger;

struct SerialLogger;

pub fn init() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(parse_level_filter());
}

impl log::Log for SerialLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut buf = [0u8; 512];
            let mut writer = BufWriter::new(&mut buf);

            let _ = core::fmt::write(
                &mut writer,
                format_args!(
                    "{}[{}] - {}\x1b[0m\n",
                    level_color(record.level()),
                    record.level(),
                    record.args()
                ),
            );

            printk(buf);
        }
    }

    fn flush(&self) {}
}

fn parse_level_filter() -> log::LevelFilter {
    match DEBUG_LEVEL {
        Some("trace") => log::LevelFilter::Trace,
        Some("debug") => log::LevelFilter::Debug,
        Some("info") => log::LevelFilter::Info,
        Some("warn") => log::LevelFilter::Warn,
        Some("error") => log::LevelFilter::Error,
        _ => log::LevelFilter::Off,
    }
}

fn level_color(level: log::Level) -> &'static str {
    match level {
        log::Level::Error => "\x1b[31m",
        log::Level::Warn => "\x1b[33m",
        log::Level::Info => "\x1b[32m",
        log::Level::Debug => "\x1b[34m",
        log::Level::Trace => "\x1b[90m",
    }
}

struct BufWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> BufWriter<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, pos: 0 }
    }
}

impl<'a> Write for BufWriter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();

        if self.pos + bytes.len() > self.buf.len() {
            return Err(fmt::Error);
        }

        self.buf[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
        self.pos += bytes.len();
        Ok(())
    }
}
