// TODO(aeryz): we probably wanna have sbi_call_2, sbi_call_3, etc.

#[inline(always)]
fn sbi_call3(eid: usize, fid: usize, arg0: usize, arg1: usize, arg2: usize) -> SbiRet {
    let error: isize;
    let value: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("a0") arg0 => error,
            inlateout("a1") arg1 => value,
            in("a2") arg2,
            in("a6") fid,
            in("a7") eid,
        );
    }
    SbiRet { error, value }
}

#[inline(always)]
fn sbi_call2(eid: usize, fid: usize, arg0: usize, arg1: usize) -> SbiRet {
    let error: isize;
    let value: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("a0") arg0 => error,
            inlateout("a1") arg1 => value,
            in("a6") fid,
            in("a7") eid,
        );
    }
    SbiRet { error, value }
}

#[inline(always)]
fn sbi_call1(eid: usize, fid: usize, arg0: usize) -> SbiRet {
    let error: isize;
    let value: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("a0") arg0 => error,
            in("a6") fid,
            in("a7") eid,
            out("a1") value,
        );
    }
    SbiRet { error, value }
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
    sbi_call3(SBI_EXT_HSM, SBI_HSM_HART_START, hartid, start_addr, opaque)
}

pub fn console_putchar(c: u8) {
    let _ = sbi_call3(0x01, 0, c as usize, 0, 0);
}

pub fn set_timer(time_val: usize) {
    let _ = sbi_call1(0x0, 0, time_val);
}
