use core::arch::asm;

use crate::userspace::syscalls;

#[unsafe(no_mangle)]
pub extern "C" fn shell() {
    unsafe { asm!(".align 12") };

    loop {
        let mut buf: [u8; 512] = [0; 512];

        let mut pos = 0;

        while buf[pos] != b'\n' {
            let n_read = syscalls::read(buf[pos..].as_mut_ptr(), 1) as usize;
            if n_read != 0 {
                pos += n_read;
            }
        }

        syscalls::write(buf.as_ptr(), pos);
    }
}
