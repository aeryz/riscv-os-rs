use bitflags::bitflags;

#[repr(C)]
pub struct Descriptor {
    /// Address (guest, physical)
    addr: u64,
    /// Length
    len: u32,
    /// The flags as indicated above.
    flags: DescriptorFlag,
    /// Next field if flags & NEXT */
    next: u16,
}

bitflags! {
    pub struct DescriptorFlag: u16 {
        /// This marks a buffer as continuing via the next field.
        const NEXT = 1 << 0;
        /// This marks a buffer as device write-only (otherwise device read-only).
        const WRITE = 1 << 1;
        /// This means the buffer contains a list of buffer descriptors.
        const INDIRECT = 1 << 2;
    }
}

#[repr(C)]
pub struct AvailableRing<const QUEUE_SIZE: usize> {
    flags: AvailableRingFlag,
    idx: u16,
    ring: [u16; QUEUE_SIZE],
    /// Only if VIRTIO_F_EVENT_IDX
    used_event: u16,
}

bitflags! {
    pub struct AvailableRingFlag: u16 {
        const NO_INTERRUPT = 1 << 0;
    }
}

#[repr(C)]
pub struct UsedRing<const QUEUE_SIZE: usize> {
    flags: UsedRingFlag,
    idx: u16,
    used_elem_ring: [UsedElemRing; QUEUE_SIZE],
    /// Only if VIRTIO_F_EVENT_IDX
    avail_event: u16,
}

bitflags! {
    pub struct UsedRingFlag: u16 {
        const NO_NOTIFY = 1 << 0;
    }
}
