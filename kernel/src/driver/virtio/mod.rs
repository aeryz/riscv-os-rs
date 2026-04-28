use bitflags::bitflags;

use crate::driver::virtio::mmio::RegisterOffset;

pub mod block;
pub mod mmio;
mod virtqueue;

const VIRTIO_F_VERSION_1: u32 = 1 << 0;
const VIRTIO0: usize = 0xffffffd61000_1000;
const VIRTIO_STRIDE: usize = 0x1000;
const VIRTIO_COUNT: usize = 8;

pub fn find_virtio_blk() -> Option<usize> {
    for i in 0..VIRTIO_COUNT {
        let base = VIRTIO0 + i * VIRTIO_STRIDE;

        let magic = mmio::read32(base, RegisterOffset::Magic);
        let version = mmio::read32(base, RegisterOffset::Version);
        let device_id = mmio::read32(base, RegisterOffset::DeviceId);

        if magic == 0x7472_6976 && version == 2 && device_id == 2 {
            return Some(base);
        }
    }

    None
}

bitflags! {
    #[repr(transparent)]
    pub struct Status: u32 {
        /// Indicates that the guest OS has found the device and recognized it as a
        /// valid virtio device.
        const ACK = 1;
        /// Indicates that the guest OS knows how to drive the device. Note: There
        /// could be a significant (or infinite) delay before setting this bit.
        /// For example, under Linux, drivers can be loadable modules.
        const DRIVER = 2;
        /// Indicates that something went wrong in the guest, and it has given up on
        /// the device. This could be an internal error, or the driver didn’t like
        /// the device for some reason, or even a fatal error during device
        /// operation.
        const FAILED = 128;
        /// Indicates that the driver has acknowledged all the features it
        /// understands, and feature negotiation is complete.
        const FEATURES_OK = 8;
        /// Indicates that the driver is set up and ready to drive the device.
        const DRIVER_OK = 4;
        /// Indicates that the device has experienced an error from which it can’t
        /// recover.
        const DEVICE_NEEDS_RESET = 64;
    }
}

impl Into<u32> for Status {
    fn into(self) -> u32 {
        self.bits() as u32
    }
}
