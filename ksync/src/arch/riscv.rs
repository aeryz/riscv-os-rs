use core::arch::asm;

pub fn atomic_test_and_set(flag: *mut usize, new_value: usize) -> usize {
    let old_value: usize;
    unsafe {
        asm!(
            "amoswap.w.aq t0, t1, (a0)",
            in("a0") flag,
            in("t1") new_value,
            lateout("t0") old_value,
            options(
                nomem,
                nostack,
            )
        );
    }

    old_value
}

pub fn atomic_set(flag: *mut usize, new_value: usize) {
    let _ = atomic_test_and_set(flag, new_value);
}
