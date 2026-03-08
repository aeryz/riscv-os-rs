use crate::impl_control_register;

#[repr(transparent)]
pub struct Mideleg(u64);

impl Mideleg {
    pub fn delegate_all(mut self) -> Self {
        self.0 = u64::MAX;
        self
    }
}

impl_control_register!(Mideleg, mideleg);
