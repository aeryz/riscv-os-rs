pub mod directory;
mod file;
mod inode;

pub use file::*;
pub use inode::*;

use crate::driver::virtio::{self, block};
use vsfs::SuperBlock;

pub fn init() {
    let mut data = &mut [0; 512];
    if unsafe { virtio::block::read(&mut data, 0) } != block::VirtioBlkStatus::Ok as u8 {
        panic!("block read failed");
    }

    let sb = data.as_ptr() as *const _ as *const SuperBlock;
    unsafe {
        log::info!("Superblock: {:?}", *sb);
    }
}
