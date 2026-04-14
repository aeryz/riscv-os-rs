use crate::impl_control_register;

#[repr(transparent)]
pub struct Medeleg(usize);

impl Medeleg {
    pub fn delegate_all(mut self) -> Self {
        self.0 = usize::MAX;
        self
    }
}

impl_control_register!(Medeleg, medeleg);
