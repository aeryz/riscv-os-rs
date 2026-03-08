use crate::impl_control_register;

#[repr(transparent)]
pub struct Medeleg(u64);

impl Medeleg {
    pub fn delegate_all(mut self) -> Self {
        self.0 = u64::MAX;
        self
    }
}

impl_control_register!(Medeleg, medeleg);
