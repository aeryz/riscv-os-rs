// TODO(aeryz): we probably wanna have sbi_call_2, sbi_call_3, etc.

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

pub const SBI_EXT_HSM: usize = 0x48534D;
pub const SBI_HSM_HART_START: usize = 0;

#[repr(C)]
pub struct SbiRet {
    pub error: isize,
    pub value: isize,
}

#[inline(always)]
pub fn hart_start(hartid: usize, start_addr: usize, opaque: usize) -> SbiRet {
    let error: isize;
    let value: isize;

    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("a0") hartid => error,
            inlateout("a1") start_addr => value,
            in("a2") opaque,
            in("a6") SBI_HSM_HART_START,
            in("a7") SBI_EXT_HSM,
            options(nostack),
        );
    }

    SbiRet { error, value }
}

pub fn console_putchar(c: u8) {
    sbi_call(0x01, 0, c as usize, 0, 0);
}
