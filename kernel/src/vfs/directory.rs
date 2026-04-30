pub struct Directory {
    data: [Option<Data>; 10],
}

#[repr(C)]
pub struct Data {
    inode: u32,
    reclen: u32,
    strlen: u32,
    name: [u8; 96],
}
