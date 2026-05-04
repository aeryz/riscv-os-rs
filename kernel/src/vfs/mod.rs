pub mod directory;
mod file;
mod inode;

use core::{cell::OnceCell, mem::MaybeUninit};

use alloc::sync::Arc;
pub use file::*;
pub use inode::*;

use crate::driver::virtio::{self, block};
use vsfs::{DirEnt, INode, SuperBlock, Type};

const MAX_DIRENTS_IN_SECTOR: usize = 512 / size_of::<DirEnt>();

static SUPERBLOCK: SuperBlockSend = SuperBlockSend(OnceCell::new());

struct SuperBlockSend(OnceCell<SuperBlock>);

// This will only be initialized once by a single core/thread.
unsafe impl Send for SuperBlockSend {}
unsafe impl Sync for SuperBlockSend {}

pub fn init() {
    SUPERBLOCK.0.get_or_init(|| {
        let mut buf = &mut [0; 512];
        if unsafe { virtio::block::read(&mut buf, 0) } != block::VirtioBlkStatus::Ok as u8 {
            panic!("block read failed");
        }

        let sb = unsafe {
            (buf.as_ptr() as *const _ as *const SuperBlock)
                .as_ref()
                .unwrap()
        };

        log::trace!("Superblock: {:?}", sb);
        *sb
    });
}

/// Open a file
///
/// TODO(aeryz): Right now, this only allows opening a file, no directory read
/// support yet.
/// TODO(aeryz): This only supports absolute paths right now
pub fn open(path: &[u8]) -> Option<File> {
    let mut current_inode = get_inode(1);
    for path in path.split(|b| *b == b'/').filter(|p| !p.is_empty()) {
        if current_inode.ty == Type::File {
            return None;
        }
        current_inode = lookup_path(&current_inode, path).unwrap();
    }

    Some(File {
        inode: Arc::new(current_inode),
        perm: FileFlag::all(),
        offset: 0,
    })
}

fn get_inode(inum: usize) -> INode {
    let sb = SUPERBLOCK.0.get().unwrap();

    let inode_byte_offset = sb.inode_table_start as usize * 4096 + inum * size_of::<INode>();
    let inode_sector = inode_byte_offset / 512;
    let inode_offset = inode_byte_offset % 512;

    let mut buf = &mut [0; 512];
    if unsafe { virtio::block::read(&mut buf, inode_sector as u64) }
        != block::VirtioBlkStatus::Ok as u8
    {
        panic!("block read failed");
    }

    unsafe { (*(buf[inode_offset..].as_ptr() as *const _ as *const INode)).clone() }
}

/*

for (block_idx, block) in direct_blocks:
    sector = block * 4096 / 512;

    for sector_idx in 0..8 {
        buf = virtio::read(sector); /* contains n dirents */

        for cur_dir_idx in 0..MAX_DIRENTS_IN_SECTOR {

            if block_idx * 8 + (sector_idx * MAX_DIRENTS_IN_SECTOR) + cur_dir_idx > n_dirent_in_inode {
                // we reached the max
                break;
            }
        }


        sector += 512;
    }

*/

/// Lookup the `path` inside `inode`. `inode` needs to be a directory.
fn lookup_path(inode: &INode, path: &[u8]) -> Result<INode, ()> {
    if inode.ty != Type::Directory {
        return Err(());
    }

    let n_dirent_in_node = inode.metadata.sz as usize / size_of::<DirEnt>();

    for (block_idx, block) in inode.direct_blocks.iter().enumerate() {
        let mut sector = block * 4096 / 512;

        for sector_idx in 0..8 {
            let mut buf = &mut [0; 512];
            unsafe {
                // TODO(aeryz): implement a macro or fn for this
                if virtio::block::read(&mut buf, sector as u64) != block::VirtioBlkStatus::Ok as u8
                {
                    return Err(());
                }
            }

            for cur_dir_idx in 0..MAX_DIRENTS_IN_SECTOR {
                if (block_idx * 8 + sector_idx) * MAX_DIRENTS_IN_SECTOR + cur_dir_idx
                    >= n_dirent_in_node
                {
                    return Err(());
                }

                let dirent = unsafe {
                    (buf[(size_of::<DirEnt>() * cur_dir_idx)..].as_ptr() as *const _
                        as *const DirEnt)
                        .as_ref()
                        .unwrap()
                };

                if &dirent.name[0..dirent.name_len as usize] == path {
                    return Ok(get_inode(dirent.inum as usize));
                }
            }

            sector += 1;
        }
    }

    Err(())
}
