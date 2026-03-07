pub mod satp;

pub trait ControlRegister {
    const NAME: &str;

    fn new(reg: u64) -> Self;

    fn read() -> Self;

    fn write(self);
}

#[macro_export]
macro_rules! impl_control_register {
    ($ty:ty, $csr:ident) => {
        impl crate::registers::ControlRegister for $ty {
            const NAME: &'static str = core::stringify!($csr);

            fn new(reg: u64) -> Self {
                Self(reg)
            }

            fn read() -> Self {
                let value: u64;
                unsafe {
                    core::arch::asm!(
                        core::concat!("csrr {}, ", core::stringify!($csr)),
                        out(reg) value,
                        options(nomem, nostack, preserves_flags),
                    );
                }
                Self::new(value)
            }

            fn write(self) {
                unsafe {
                    core::arch::asm!(
                        core::concat!("csrw ", core::stringify!($csr), ", {}"),
                        in(reg) self.0,
                        options(nomem, nostack, preserves_flags)
                    );
                }
            }
        }
    };
}
