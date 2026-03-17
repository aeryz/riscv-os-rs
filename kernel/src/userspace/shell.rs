use core::arch::asm;

use crate::userspace::syscalls;

const BINS: &[&[u8]] = &[CMD_HELP];
const PROMPT: &[u8] = b"shell $ ";

const CMD_HELP: &[u8] = b"help";

#[unsafe(no_mangle)]
pub extern "C" fn shell() {
    unsafe { asm!(".align 12") };

    loop {
        core::hint::spin_loop();
    }

    loop {
        let mut buf: [u8; 512] = [0; 512];

        let mut pos = 0;

        super::write(PROMPT);

        while buf[pos] != b'\n' && buf[pos] != b'\r' {
            let n_read = syscalls::read(buf[pos..].as_mut_ptr(), 1) as usize;
            if n_read == 0 {
                super::write("\n");
                break;
            }
            match buf[pos] {
                127 | 8 => {
                    if pos > 0 {
                        buf[pos] = 0;
                        pos -= 1;

                        super::write(b"\x08 \x08");
                    }
                }
                _ => {
                    if pos >= buf.len() {
                        break;
                    }
                    let _ = super::write(&buf[pos..]);
                    pos += 1;
                }
            }
        }

        if let Some(binary) = BINS.iter().find(|bin| **bin == &buf[0..pos]) {
            match *binary {
                CMD_HELP => {
                    let cmds = "available commands:\n";
                    super::write(cmds);
                    BINS.iter().for_each(|b| {
                        super::write("- ");
                        super::write(b);
                        super::write("\n");
                    })
                }
                _ => {}
            }
        } else {
            let msg = "shell: command not found: ";
            super::write(msg);
            super::write(&buf[0..pos]);
            super::write(b"\n");
        }
    }
}
