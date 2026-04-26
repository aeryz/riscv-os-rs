#![allow(unused)]

use alloc::vec::Vec;
use ksync::RwLock;

use crate::driver::virtio::mmio;

use super::Status;

static DRIVER: RwLock<VirtioBlkDriver> = RwLock::new(VirtioBlkDriver {
    virtqueue: Vec::new(),
});

pub struct VirtioBlkDriver {
    virtqueue: Vec<u8>,
}

// TODO(aeryz): virtual address
// TODO(aeryz): most of the stuff being done here are device
/// 3. Set the DRIVER status bit: the guest OS knows how to drive the device.
pub fn init(device_base: usize) -> Result<(), ()> {
    mmio::init_device(device_base, 0, || {
        // - Drivers SHOULD NOT negotiate VIRTIO_BLK_F_FLUSH if they are
        //   incapable of sending VIRTIO_BLK_T_FLUSH commands.
        //
        // - If neither VIRTIO_BLK_F_CONFIG_WCE nor VIRTIO_BLK_F_FLUSH are
        //   negotiated, the driver MAY deduce the presence of a writethrough
        //   cache. If VIRTIO_BLK_F_CONFIG_WCE was not negotiated but
        //   VIRTIO_BLK_F_FLUSH was, the driver SHOULD assume presence of a
        //   writeback cache.
        //
        // - The driver MUST NOT read writeback before setting the FEATURES_OK
        //   device status bit.
        //
        // - Drivers MUST NOT negotiate the VIRTIO_BLK_F_ZONED feature if they
        //   are incapable of supporting devices with the VIRTIO_BLK_Z_HM,
        //   VIRTIO_BLK_Z_HA or VIRTIO_BLK_Z_NONE zoned model.
        //
        // - If the VIRTIO_BLK_F_ZONED feature is offered by the device with the
        // VIRTIO_BLK_Z_HM zone model, then the VIRTIO_BLK_F_DISCARD feature
        // MUST NOT be offered by the driver.
        //
        // - If the VIRTIO_BLK_F_ZONED feature and VIRTIO_BLK_F_DISCARD feature
        // are both offered by the device with the VIRTIO_BLK_Z_HA or
        // VIRTIO_BLK_Z_NONE zone model, then the driver MAY negotiate these two
        // bits independently.

        // - If the VIRTIO_BLK_F_ZONED feature is negotiated, then
        //
        //     a. if the driver that can not support host-managed zoned devices
        // reads VIRTIO_BLK_Z_HM from the model field of zoned, the driver MUST
        // NOT set FEATURES_OK flag and instead set the FAILED bit.
        //     b. if the driver that can not support zoned devices reads
        // VIRTIO_BLK_Z_HA from the model field of zoned, the driver MAY handle
        // the device as a non-zoned device. In this case, the driver SHOULD
        // ignore all other fields in zoned.
    })?;

    // Virtqueue configuration
    // 1. Select the queue by writing its index to QueueSel. Select the queue by
    // writing its index to QueueSel.
    mmio::write32(device_base, mmio::RegisterOffset::QueueSel, 0u32);
    // 2. Check if the queue is not already in use: read QueueReady, and expect a
    // returned value of zero (0x0).
    if mmio::read32(device_base, mmio::RegisterOffset::QueueReady) != 0 {
        return Err(());
    }
    // 3. Read maximum queue size (number of elements) from QueueSizeMax. If the
    // returned value is zero (0x0) the queue is not available.
    match mmio::read32(device_base, mmio::RegisterOffset::QueueSizeMax) {
        0 => return Err(()),
        n => {
            log::debug!("virtqueue size max: {n}");
        }
    }
    // 4. Allocate and zero the queue memory, making sure the memory is physically
    // contiguous.
    let virtqueue = Vec::with_capacity(1024);
    DRIVER.write_lock().virtqueue = virtqueue;
    // 5. Notify the device about the queue size by writing the size to QueueSize.
    // 6. Write physical addresses of the queue’s Descriptor Area, Driver Area and
    // Device Area to (respectively) the QueueDescLow/QueueDescHigh,
    // QueueDriverLow/QueueDriverHigh and QueueDeviceLow/QueueDeviceHigh register
    // pairs.
    // 7. Write 0x1 to QueueReady.

    Ok(())
}

pub fn operate() {
    let mut data = [0u8; 512];
    let hello = b"helloworldomg";
    data[0..hello.len()].copy_from_slice(hello);
    VirtioBlkReq {
        ty: VIRTIO_BLK_T_OUT,
        _reserved: 0,
        sector: 1,
        data: (&mut data) as *mut u8,
        status: todo!(),
    };
}

/// Read
const VIRTIO_BLK_T_IN: u32 = 0;
/// Write
const VIRTIO_BLK_T_OUT: u32 = 1;
/// Flush
const VIRTIO_BLK_T_FLUSH: u32 = 4;
/// Get device ID
/// Fetches the device ID string from the device into data.
/// The device ID string is a NUL-padded ASCII string up to 20 bytes long.
/// If the string is 20 bytes long then there is no NUL terminator.
const VIRTIO_BLK_T_GET_ID: u32 = 8;
/// Get the device lifetime // TODO(aeryz): what's the device lifetime
/// The data used for VIRTIO_BLK_T_GET_LIFETIME requests is populated by the
/// device, and is of the form [`VirtioBlkLifetime`]
const VIRTIO_BLK_T_GET_LIFETIME: u32 = 10;
/// Discard
const VIRTIO_BLK_T_DISCARD: u32 = 11;
/// Fill with zeroes
const VIRTIO_BLK_T_WRITE_ZEROES: u32 = 13;
/// Secure erase: TODO(aeryz): what's this
const VIRTIO_BLK_T_SECURE_ERASE: u32 = 14;

#[repr(C)]
/// The following
/// The driver enqueues requests to the virtqueues, and they are used by the
/// device (not necessarily in order). Each request except
/// VIRTIO_BLK_T_ZONE_APPEND is of form:
pub struct VirtioBlkReq {
    ty: u32,
    _reserved: u32,
    /// Indicates the offset (multiplied by 512) where the read or write is to
    /// occur. This field is unused and set to 0 for commands other than
    /// read, write and some zone operations
    sector: u64,
    /// VIRTIO_BLK_T_IN requests populate data with the contents of sectors read
    /// from the block device (in multiples of 512 bytes). VIRTIO_BLK_T_OUT
    /// requests write the contents of data to the block device (in
    /// multiples of 512 bytes).
    data: *mut u8,
    status: u8,
}

/// The data used for discard, secure erase or write zeroes commands consists of
/// one or more segments. The maximum number of segments is max_discard_seg for
/// discard commands, max_secure_erase_seg for secure erase commands and
/// max_write_zeroes_seg for write zeroes commands. Each segment is of form:
pub struct VirtioBlkDiscardWriteZeroes {
    /// indicates the starting offset (in 512-byte units) of the segment
    sector: u64,
    /// indicates the number of sectors in each discarded range
    num_sectors: u32,
    /// only used in write zeroes commands and allows the device to discard the
    /// specified range, provided that following reads return zeroes.
    // struct {
    //         le32 unmap:1;
    //         le32 reserved:31;
    // } flags;
    flags: u64,
}

pub struct VirtioBlkLifetime {
    /// specifies the percentage of reserved blocks that are consumed
    pre_eol_info: u16,
    /// refers to wear of SLC cells and is provided in increments of
    /// 10used, and so on, thru to 11 meaning estimated lifetime exceeded.
    /// All values above 11 are reserved.
    device_lifetime_est_typ_a: u16,
    /// refers to wear of MLC cells and is provided with the same semantics
    /// as device_lifetime_est_typ_a.
    device_lifetime_est_typ_b: u16,
}

// Note: In the legacy interface, VIRTIO_BLK_F_FLUSH was also called
// VIRTIO_BLK_F_WCE.
/// LEGACY: Device supports request barriers.
const VIRTIO_BLK_F_BARRIE: u32 = 1 << 0;
/// LEGACY: Device supports scsi packet commands.
const VIRTIO_BLK_F_SCSI: u32 = 1 << 7;

/// Value not available
const VIRTIO_BLK_PRE_EOL_INFO_UNDEFINED: u16 = 0;
/// < 80% of reserved blocks are consumed
const VIRTIO_BLK_PRE_EOL_INFO_NORMAL: u16 = 1;
/// 80% of reserved blocks are consumed
const VIRTIO_BLK_PRE_EOL_INFO_WARNING: u16 = 2;
/// 90% of reserved blocks are consumed
const VIRTIO_BLK_PRE_EOL_INFO_URGENT: u16 = 3;

/// The final status byte is written by the device: either VIRTIO_BLK_S_OK for
/// success, VIRTIO_BLK_S_IOERR for device or driver error or
/// VIRTIO_BLK_S_UNSUPP for a request unsupported by device:
// TODO(aeryz): dunno what's the type of these
const VIRTIO_BLK_S_OK: u32 = 0;
const VIRTIO_BLK_S_IOERR: u32 = 1;
const VIRTIO_BLK_S_UNSUPP: u32 = 2;

/// Maximum size of any single segment is in size_max.
pub const VIRTIO_BLK_F_SIZE_MAX: u32 = 1 << 1;
/// Maximum number of segments in a request is in seg_max.
pub const VIRTIO_BLK_F_SEG_MAX: u32 = 1 << 2;
/// Disk-style geometry specified in geometry.
pub const VIRTIO_BLK_F_GEOMETRY: u32 = 1 << 4;
/// Device is read-only.
pub const VIRTIO_BLK_F_RO: u32 = 1 << 5;
/// Block size of disk is in blk_size.
pub const VIRTIO_BLK_F_BLK_SIZE: u32 = 1 << 6;
/// Cache flush command support.
pub const VIRTIO_BLK_F_FLUSH: u32 = 1 << 9;
/// Device exports information on optimal I/O alignment.
pub const VIRTIO_BLK_F_TOPOLOGY: u32 = 1 << 10;
/// Device can toggle its cache between writeback and writethrough modes.
pub const VIRTIO_BLK_F_CONFIG_WCE: u32 = 1 << 11;
/// Device supports multiqueue.
pub const VIRTIO_BLK_F_MQ: u32 = 1 << 12;
/// Device can support discard command, maximum discard sectors size in
/// max_discard_sectors and maximum discard segment number in max_discard_seg.
pub const VIRTIO_BLK_F_DISCARD: u32 = 1 << 13;
/// Device can support write zeroes command, maximum write zeroes sectors size
/// in max_write_zeroes_sectors and maximum write zeroes segment number in
/// max_write_zeroes_seg.
pub const VIRTIO_BLK_F_WRITE_ZEROES: u32 = 1 << 14;
/// Device supports providing storage lifetime information.
pub const VIRTIO_BLK_F_LIFETIME: u32 = 1 << 15;
/// Device supports secure erase command, maximum erase sectors count in
/// max_secure_erase_sectors and maximum erase segment number in
/// max_secure_erase_seg.
pub const VIRTIO_BLK_F_SECURE_ERASE: u32 = 1 << 16;
/// Device is a Zoned Block Device, that is, a device that follows the zoned
/// storage device behavior that is also supported by industry standards such as
/// the T10 Zoned Block Command standard (ZBC r05) or the NVMe(TM) NVM Express
/// Zoned Namespace Command Set Specification 1.1b (ZNS). For brevity, these
/// standard documents are referred as "ZBD standards" from this point on in the
/// text.
pub const VIRTIO_BLK_F_ZONED: u32 = 1 << 17;

struct VirtioBlkGeometry {
    cylinders: u16,
    heads: u8,
    sectors: u8,
}

struct VirtioBlkTopology {
    /// # of logical blocks per physical block (log2)
    physical_block_exp: u8,
    /// offset of first aligned logical block
    alignment_offset: u8,
    /// suggested minimum I/O size in blocks
    min_io_size: u16,
    /// optimal (suggested maximum) I/O size in blocks
    opt_io_size: u32,
}

pub struct VirtioBlkConfig {
    /// The capacity of the device (expressed in 512-byte sectors) is always
    /// present. The availability of the others all depend on various
    /// feature bits as indicated above.
    capacity: u64,
    size_max: u32,
    seg_max: u32,
    geometry: VirtioBlkGeometry,
    blk_size: u32,
    topology: VirtioBlkTopology,
    writeback: u8,
    unused0: u8,
    num_queues: u16,
    max_discard_sectors: u32,
    max_discard_seg: u32,
    discard_sector_alignment: u32,
    max_write_zeroes_sectors: u32,
    max_write_zeroes_seg: u32,
    write_zeroes_may_unmap: u8,
    _unused1: [u8; 3],
    max_secure_erase_sectors: u32,
    max_secure_erase_seg: u32,
    secure_erase_sector_alignment: u32,
    zoned: VirtioBlkZonedCharacteristics,
}

struct VirtioBlkZonedCharacteristics {
    zone_sectors: u32,
    max_open_zones: u32,
    max_active_zones: u32,
    max_append_sectors: u32,
    write_granularity: u32,
    model: u8,
    _unused2: [u8; 3],
}
