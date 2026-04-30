use alloc::sync::Arc;
use bitflags::bitflags;
use ksync::RwLock;

use crate::vfs::INode;

bitflags! {
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

/// File reference that is created per task. References to a global file (inode).
pub struct File {
    inode: Arc<INode>,
    perm: FileFlag,
    offset: usize,
}

pub struct FileDescriptor {
    file: Arc<RwLock<File>>,
}
