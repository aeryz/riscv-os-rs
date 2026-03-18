pub mod shell;
pub mod syscalls;

#[inline(always)]
pub fn write<T: AsRef<[u8]>>(buf: T) -> isize {
    syscalls::write(buf.as_ref().as_ptr(), buf.as_ref().len())
}

#[unsafe(no_mangle)]
pub extern "C" fn userspace_1() -> ! {
    unsafe { core::arch::asm!(".align 12") };
    let mut i = 0u64;
    loop {
        i += 1;

        if i % 100_000_000 == 0 {
            let _ = write("[1] writing babeee");
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn userspace_2() -> ! {
    unsafe { core::arch::asm!(".align 12") };
    let mut i = 0u64;
    loop {
        i += 1;

        if i % 3_000_000_000 == 0 {
            let _ = write("[2] writing babeee");
        }
    }
}
