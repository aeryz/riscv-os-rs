pub mod shell;
pub mod syscalls;

#[inline(always)]
pub fn write<T: AsRef<[u8]>>(buf: T) -> isize {
    syscalls::write(buf.as_ref().as_ptr(), buf.as_ref().len())
}

#[unsafe(no_mangle)]
pub extern "C" fn userspace_sleep_print_loop_1() -> ! {
    unsafe { core::arch::asm!(".align 12") };
    loop {
        let _ = write("[task-1] writing babeee\n");
        syscalls::sleep_ms(1200);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn userspace_sleep_print_loop_2() -> ! {
    unsafe { core::arch::asm!(".align 12") };
    loop {
        let _ = write("[task-2] writing babeee\n");
        syscalls::sleep_ms(2700);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn userspace_sleep_print_loop_3() -> ! {
    unsafe { core::arch::asm!(".align 12") };
    loop {
        let _ = write("[task-3] writing babeee\n");
        syscalls::sleep_ms(1300);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn userspace_sleep_print_loop_4() -> ! {
    unsafe { core::arch::asm!(".align 12") };
    loop {
        let _ = write("[task-4] writing babeee\n");
        syscalls::sleep_ms(2200);
    }
}
