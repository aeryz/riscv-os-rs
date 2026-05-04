use alloc::sync::Arc;
use bitflags::bitflags;
use ksync::RwLock;

use crate::vfs::INode;

bitflags! {
    #[derive(Debug)]
    pub struct FileFlag: usize {
        const R = 1 << 0;
        const W = 1 << 1;
        const X = 1 << 2;
        const RW = (1 << 0) | (1 << 1);
        const RX = (1 << 2) | (1 << 1);
        const RWX = (1 << 0) | (1 << 1) | (1 << 2);
        const WX = (1 << 1) | (1 << 2);
    }
}

/// File reference that is created per task. References to a global file
/// (inode).
#[derive(Debug)]
pub struct File {
    pub inode: Arc<INode>,
    pub perm: FileFlag,
    pub offset: usize,
}

pub struct FileDescriptor {
    pub file: Arc<RwLock<File>>,
}
