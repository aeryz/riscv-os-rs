pub mod directory;
mod file;
mod inode;

use core::mem::MaybeUninit;

pub use file::*;
pub use inode::*;

use crate::driver::virtio::{self, block};
use vsfs::{DirEnt, INode, SuperBlock, Type};

const ROOT_INO: usize = 1;

pub fn init() {
    let mut buf = &mut [0; 512];
    let (superblock, root_inode_ptr) = {
        if unsafe { virtio::block::read(&mut buf, 0) } != block::VirtioBlkStatus::Ok as u8 {
            panic!("block read failed");
        }

        let sb = unsafe {
            (buf.as_ptr() as *const _ as *const SuperBlock)
                .as_ref()
                .unwrap()
        };
        log::info!("Superblock: {:?}", sb);

        (
            *sb,
            sb.inode_table_start as usize * 4096 + ROOT_INO * size_of::<INode>(),
        )
    };

    let sector = root_inode_ptr / 512;
    let offset = root_inode_ptr % 512;

    let (root_data_sector, n_dirents) = {
        if unsafe { virtio::block::read(&mut buf, sector as u64) }
            != block::VirtioBlkStatus::Ok as u8
        {
            panic!("block read failed");
        }

        log::info!("Reading from the sector: {sector} with offset: {offset}");

        let inode = unsafe {
            (buf[offset..].as_ptr() as *const _ as *const INode)
                .as_ref()
                .unwrap()
        };

        if inode.link_count == 0 || inode.ty != Type::Directory {
            panic!("the root directory is a valid directory");
        }

        (
            inode.direct_blocks[0] as usize * 4096 / 512,
            inode.metadata.sz as usize / size_of::<DirEnt>(),
        )
    };

    if unsafe { virtio::block::read(&mut buf, root_data_sector as u64) }
        != block::VirtioBlkStatus::Ok as u8
    {
        panic!("block read failed");
    }

    let foo_data = (0..n_dirents)
        .find_map(|i| {
            let entry = unsafe {
                (buf.as_ptr().byte_offset((size_of::<DirEnt>() * i) as isize) as *const _
                    as *const DirEnt)
                    .as_ref()
                    .unwrap()
            };
            log::info!("entry: {entry:?}");

            if &entry.name[0..entry.len as usize] == b"foo" {
                Some(entry)
            } else {
                None
            }
        })
        .expect("couldn't find foo");

    let foo_inode_ptr =
        superblock.inode_table_start as usize * 4096 + foo_data.inum as usize * size_of::<INode>();
    let sector = foo_inode_ptr / 512;
    let offset = foo_inode_ptr % 512;

    log::error!("foo inode ptr: {foo_inode_ptr} sector: {sector} offset: {offset}");

    let (foo_data_sector, n_dirents) = {
        if unsafe { virtio::block::read(&mut buf, sector as u64) }
            != block::VirtioBlkStatus::Ok as u8
        {
            panic!("block read failed");
        }

        log::info!("Reading from the sector: {sector} with offset: {offset}");

        let inode = unsafe {
            (buf[offset..].as_ptr() as *const _ as *const INode)
                .as_ref()
                .unwrap()
        };

        log::error!("foo inode: {inode:?}");

        if inode.link_count == 0 || inode.ty != Type::Directory {
            panic!("foo directory is a valid directory");
        }

        (
            inode.direct_blocks[0] as usize * 4096 / 512,
            inode.metadata.sz as usize / size_of::<DirEnt>(),
        )
    };

    if unsafe { virtio::block::read(&mut buf, foo_data_sector as u64) }
        != block::VirtioBlkStatus::Ok as u8
    {
        panic!("block read failed");
    }

    let bar_data = (0..n_dirents)
        .find_map(|i| {
            let entry = unsafe {
                (buf.as_ptr().byte_offset((size_of::<DirEnt>() * i) as isize) as *const _
                    as *const DirEnt)
                    .as_ref()
                    .unwrap()
            };
            log::info!("entry: {entry:?}");

            if &entry.name[0..entry.len as usize] == b"bar" {
                Some(entry)
            } else {
                None
            }
        })
        .expect("couldn't find foo");

    let bar_inode_ptr =
        superblock.inode_table_start as usize * 4096 + bar_data.inum as usize * size_of::<INode>();
    let sector = bar_inode_ptr / 512;
    let offset = bar_inode_ptr % 512;

    log::error!("bar inode ptr: {bar_inode_ptr} sector: {sector} offset: {offset}");

    let (bar_data_sector, data_sz) = {
        if unsafe { virtio::block::read(&mut buf, sector as u64) }
            != block::VirtioBlkStatus::Ok as u8
        {
            panic!("block read failed");
        }

        log::info!("Reading from the sector: {sector} with offset: {offset}");

        let inode = unsafe {
            (buf[offset..].as_ptr() as *const _ as *const INode)
                .as_ref()
                .unwrap()
        };

        log::error!("bar inode: {inode:?}");

        if inode.link_count == 0 || inode.ty != Type::File {
            panic!("bar is a valid file");
        }

        (
            inode.direct_blocks[0] as usize * 4096 / 512,
            inode.metadata.sz,
        )
    };

    if unsafe { virtio::block::read(&mut buf, bar_data_sector as u64) }
        != block::VirtioBlkStatus::Ok as u8
    {
        panic!("block read failed");
    }

    unsafe {
        panic!(
            "data: {}",
            str::from_utf8_unchecked(&buf[0..data_sz as usize])
        );
    }
}
