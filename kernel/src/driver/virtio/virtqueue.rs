use bitflags::bitflags;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Descriptor {
    /// Address (guest, physical)
    pub(crate) addr: u64,
    /// Length
    pub(crate) len: u32,
    /// The flags as indicated above.
    pub(crate) flags: DescriptorFlag,
    /// Next field if flags & NEXT */
    pub(crate) next: u16,
}

bitflags! {
    #[derive(Debug, Clone)]
    #[repr(transparent)]
    pub struct DescriptorFlag: u16 {
        /// This marks a buffer as continuing via the next field.
        const NEXT = 1 << 0;
        /// This marks a buffer as device write-only (otherwise device read-only).
        const WRITE = 1 << 1;
        /// This means the buffer contains a list of buffer descriptors.
        const INDIRECT = 1 << 2;
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct AvailableRing<const QUEUE_SIZE: usize> {
    pub(crate) flags: AvailableRingFlag,
    pub(crate) idx: u16,
    pub(crate) ring: [u16; QUEUE_SIZE],
    // used_event: u16,
}

bitflags! {
    #[derive(Debug, Clone)]
    #[repr(transparent)]
    pub struct AvailableRingFlag: u16 {
        const NO_INTERRUPT = 1 << 0;
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct UsedRing<const QUEUE_SIZE: usize> {
    pub(crate) flags: UsedRingFlag,
    pub(crate) idx: u16,
    pub(crate) used_elem_ring: [UsedElem; QUEUE_SIZE],
    // avail_event: u16,
}

bitflags! {
    #[derive(Debug, Clone)]
    #[repr(transparent)]
    pub struct UsedRingFlag: u16 {
        const NO_NOTIFY = 1 << 0;
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct UsedElem {
    /// Index of start of used descriptor chain.
    pub(crate) id: u32,
    /// The number of bytes written into the device writable portion of
    /// the buffer described by the descriptor chain.
    pub(crate) len: u32,
}
