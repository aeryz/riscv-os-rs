use bitflags::bitflags;

#[derive(Debug)]
#[repr(C)]
pub struct INode {
    flags: Flags,
    ty: Type,
    metadata: Metadata,
    /// The first ptr always points to the data. If the file grows enough,
    /// the rest is incrementally allocated for indirect pointers.
    blocks: [u32; 12],
}

#[derive(Debug)]
#[repr(C)]
pub struct Metadata {
    /// ID of the device containing file
    // TODO(aeryz): should I have a dev_t?
    dev: u32,
    /// inode number
    ino: u32,
    /// Total size of the file in bytes
    sz: u64,
    /// Time of the last access
    access_time: u64,
    /// Time of the last modification
    modified_time: u64,
}

bitflags! {
    #[derive(Debug)]
    #[repr(transparent)]
    pub struct Flags: u32 {
        const USED = 1;
    }
}

#[derive(Debug)]
#[repr(u32)]
pub enum Type {
    Directory = 1,
    File = 2,
}
