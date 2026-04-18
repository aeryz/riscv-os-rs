pub mod shell;
pub mod syscalls;

#[inline(always)]
pub fn write<T: AsRef<[u8]>>(buf: T) -> isize {
    syscalls::write(buf.as_ref().as_ptr(), buf.as_ref().len())
}

#[unsafe(no_mangle)]
pub extern "C" fn userspace_sleep_print_loop() -> ! {
    unsafe { core::arch::asm!(".align 12") };
    loop {
        let _ = write("[1] writing babeee");
        syscalls::sleep_ms(1500);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn userspace_sleep_print_loop2() -> ! {
    unsafe { core::arch::asm!(".align 12") };
    loop {
        let _ = write("[2] writing babeee");
        syscalls::sleep_ms(3700);
    }
}
