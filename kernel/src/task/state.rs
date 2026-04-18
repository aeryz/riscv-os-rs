#![allow(unused)]

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TaskState {
    Sleeping,
    Running,
    Ready,
    Blocked,
    Zombie,
    Exited,
}
