#[macro_export]
macro_rules! impl_control_register {
    ($ty:ty, $csr:ident) => {
        impl $ty {
            pub const NAME: &'static str = core::stringify!($csr);

            #[must_use]
            pub const fn empty() -> Self {
                Self(0)
            }

            #[must_use]
            pub const fn new(reg: u64) -> Self {
                Self(reg)
            }

            #[must_use]
            pub const fn raw(&self) -> u64 {
                self.0
            }

            #[must_use]
            pub fn read() -> Self {
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

            pub fn write(self) {
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

#[macro_export]
macro_rules! def_impl_control_register {
    ($ty:ident, $csr:ident) => {
        #[repr(transparent)]
        pub struct $ty(u64);

        $crate::impl_control_register!($ty, $csr);
    };
}

#[macro_export]
macro_rules! impl_bit_set {
    ($set_fn:ident, $mask:ident) => {
        #[must_use]
        pub fn $set_fn(mut self) -> Self {
            self.0 |= Self::$mask;
            self
        }
    };
}
