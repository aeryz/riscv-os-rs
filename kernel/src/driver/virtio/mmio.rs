#![allow(unused)]

use crate::driver::virtio::Status;

pub const VIRTIO_MMIO_MAGIC: u32 = 0x7472_6976;
pub const VIRTIO_MMIO_VERSION: u32 = 0x2;

#[repr(usize)]
pub enum RegisterOffset {
    /// 0x74726976 (a Little Endian equivalent of the “virt” string).
    Magic = 0x0,
    /// 0x2. Note: Legacy devices (see 4.2.4 Legacy interface) used 0x1.
    Version = 0x4,
    /// See 5 Device Types for possible values. Value zero (0x0) is used
    /// to define a system memory map with placeholder devices at static,
    /// well known addresses, assigning functions to them depending on user’s
    /// needs.
    DeviceId = 0x8,
    /// Virtio Subsystem Vendor ID
    VendorId = 0xc,
    /// Flags representing features the device supports
    /// Reading from this register returns 32 consecutive flag bits,
    /// the least significant bit depending on the last value written
    /// to DeviceFeaturesSel. Access to this register returns bits
    /// DeviceFeaturesSel ∗ 32 to (DeviceFeaturesSel ∗ 32) + 31, eg.
    /// feature bits 0 to 31 if DeviceFeaturesSel is set to 0 and features
    /// bits 32 to 63 if DeviceFeaturesSel is set to 1. Also see 2.2 Feature
    /// Bits.
    DeviceFeatures = 0x10,
    /// Device (host) features word selection.
    /// Writing to this register selects a set of 32 device feature bits
    /// accessible by reading from DeviceFeatures.
    DeviceFeaturesSel = 0x14,
    /// Flags representing device features understood and activated by the
    /// driver Writing to this register sets 32 consecutive flag bits, the
    /// least significant bit depending on the last value written to
    /// DriverFeaturesSel. Access to this register sets bits
    /// DriverFeaturesSel ∗ 32 to (DriverFeaturesSel ∗ 32) + 31, eg. feature
    /// bits 0 to 31 if DriverFeaturesSel is set to 0 and features bits 32
    /// to 63 if DriverFeaturesSel is set to 1. Also see 2.2 Feature Bits.
    DriverFeatures = 0x20,
    /// Activated (guest) features word selection
    /// Writing to this register selects a set of 32 activated feature bits
    /// accessible by writing to DriverFeatures.
    DriverFeaturesSel = 0x24,
    /// Virtqueue index
    /// Writing to this register selects the virtqueue that the following
    /// operations on QueueSizeMax, QueueSize, QueueReady, QueueDescLow,
    /// QueueDescHigh, QueueDriverlLow, QueueDriverHigh, QueueDeviceLow,
    /// QueueDeviceHigh and QueueReset apply to.
    QueueSel = 0x30,
    /// Maximum virtqueue size
    /// Reading from the register returns the maximum size (number of elements)
    /// of the queue the device is ready to process or zero (0x0) if the
    /// queue is not available. This applies to the queue selected by
    /// writing to QueueSel. Note: QueueSizeMax was previously known as
    /// QueueNumMax.
    QueueSizeMax = 0x34,
    /// Virtqueue size
    /// Queue size is the number of elements in the queue. Writing to this
    /// register notifies the device what size of the queue the driver will
    /// use. This applies to the queue selected by writing to QueueSel.
    /// Note: QueueSize was previously known as QueueNum.
    QueueSize = 0x38,
    /// Virtqueue ready bit
    /// Writing one (0x1) to this register notifies the device that it can
    /// execute requests from this virtqueue. Reading from this register
    /// returns the last value written to it. Both read and write accesses
    /// apply to the queue selected by writing to QueueSel.
    QueueReady = 0x44,
    /// Queue notifier
    /// Writing a value to this register notifies the device that there are new
    /// buffers to process in a queue.
    QueueNotify = 0x50,
    /// Interrupt status
    /// Reading from this register returns a bit mask of events that caused the
    /// device interrupt to be asserted. The following events are possible:
    /// Used Buffer Notification
    ///    - bit 0 - the interrupt was asserted because the device has used a
    ///      buffer in at least one of the active virtqueues.
    /// Configuration Change Notification
    ///    - bit 1 - the interrupt was asserted because the configuration of the
    ///      device has changed.
    InterruptStatus = 0x60,
    /// Interrupt acknowledge
    /// Writing a value with bits set as defined in InterruptStatus to this
    /// register notifies the device that events causing the interrupt have
    /// been handled.
    InterruptAck = 0x64,
    /// Device status
    /// Reading from this register returns the current device status flags.
    /// Writing non-zero values to this register sets the status flags,
    /// indicating the driver progress. Writing zero (0x0) to this register
    /// triggers a device reset. See also p. 4.2.3.1 Device Initialization.
    Status = 0x70,
    /// Virtqueue’s Descriptor Area 64 bit long physical address
    /// Writing to these two registers (lower 32 bits of the address to
    /// QueueDescLow, higher 32 bits to QueueDescHigh) notifies the device
    /// about location of the Descriptor Area of the queue selected by
    /// writing to QueueSel register.
    QueueDescLow = 0x80,
    /// See `QueueDescLow`
    QueueDescHigh = 0x84,
    /// Virtqueue’s Driver Area 64 bit long physical address
    /// Writing to these two registers (lower 32 bits of the address to
    /// QueueDriverLow, higher 32 bits to QueueDriverHigh) notifies the
    /// device about location of the Driver Area of the queue selected by
    /// writing to QueueSel.
    QueueDriverLow = 0x090,
    /// See `QueueDriverLow`
    QueueDriverHigh = 0x094,
    /// Virtqueue’s Device Area 64 bit long physical address
    /// Writing to these two registers (lower 32 bits of the address to
    /// QueueDeviceLow, higher 32 bits to QueueDeviceHigh) notifies the
    /// device about location of the Device Area of the queue selected by
    /// writing to QueueSel.
    QueueDeviceLow = 0x0a0,
    /// See `QueueDeviceLow`
    QueueDeviceHigh = 0x0a4,
    /// Shared memory id
    /// Writing to this register selects the shared memory region 2.10 following
    /// operations on SHMLenLow, SHMLenHigh, SHMBaseLow and SHMBaseHigh apply
    /// to.
    SHMSel = 0x0ac,
    /// Shared memory region 64 bit long length
    /// These registers return the length of the shared memory region in bytes,
    /// as defined by the device for the region selected by the SHMSel register.
    /// The lower 32 bits of the length are read from SHMLenLow and the higher
    /// 32 bits from SHMLenHigh. Reading from a non-existent region (i.e. where
    /// the ID written to SHMSel is unused) results in a length of -1.
    SHMLenLow = 0x0b0,
    /// See `SHMLenLow`
    SHMLenHigh = 0x0b4,
    /// Shared memory region 64 bit long physical address
    /// The driver reads these registers to discover the base address of the
    /// region in physical address space. This address is chosen by the
    /// device (or other part of the VMM). The lower 32 bits of the address
    /// are read from SHMBaseLow with the higher 32 bits from SHMBaseHigh.
    /// Reading from a non-existent region  (i.e. where the ID written to
    /// SHMSel is unused) results in a base address of 0xffffffffffffffff.
    SHMBaseLow = 0x0b8,
    /// See `SHMBaseLow`
    SHMBaseHigh = 0x0bc,
    /// Virtqueue reset bit
    /// If VIRTIO_F_RING_RESET has been negotiated, writing one (0x1) to this
    /// register selectively resets the queue. Both read and write accesses
    /// apply to the queue selected by writing to QueueSel.
    QueueReset = 0x0c0,
    /// Configuration atomicity value
    /// Reading from this register returns a value describing a version of the
    /// device-specific configuration space (see Config). The driver can then
    /// access the configuration space and, when finished, read
    /// ConfigGeneration again. If no part of the configuration space has
    /// changed between these two ConfigGeneration reads, the returned
    /// values are identical. If the values are different, the configuration
    /// space accesses were not atomic and the driver has to perform the
    /// operations again. See also 2.5.
    ConfigGeneration = 0x0fc,
    /// Configuration space
    /// Device-specific configuration space starts at the offset 0x100 and is
    /// accessed with byte alignment. Its meaning and size depend on the
    /// device and the driver.
    Config = 0x100,
}

impl Into<u32> for RegisterOffset {
    fn into(self) -> u32 {
        self as u32
    }
}

// TODO(aeryz): We only support writing features 0..32 rn
pub fn init_device(
    device_base: usize,
    device_features: u32,
    device_init_fn: fn(),
) -> Result<(), ()> {
    // 1. Reset the device.
    write32(device_base, RegisterOffset::Status, 0u32);

    // 2. Set the ACKNOWLEDGE status bit: the guest OS has noticed the device.
    write32(device_base, RegisterOffset::Status, Status::Ack);

    write32(device_base, RegisterOffset::Status, Status::Driver);

    // 4. Read device feature bits, and write the subset of feature bits understood
    // by the OS and driver to the device. During this step the driver MAY read
    // (but MUST NOT write) the device-specific configuration fields to check that
    // it can support the device before accepting it.
    // TODO(aeryz): check if we need to support any of the features
    write32(device_base, RegisterOffset::DeviceFeatures, device_features);

    // 5. Set the FEATURES_OK status bit. The driver MUST NOT accept new feature
    // bits after this step.
    write32(device_base, RegisterOffset::Status, Status::FeaturesOk);

    // 6. Re-read device status to ensure the FEATURES_OK bit is still set:
    // otherwise, the device does not support our subset of features and the
    // device is unusable.
    if read32(device_base, RegisterOffset::Status) != Status::FeaturesOk.into() {
        return Err(());
    }

    // 7. Perform device-specific setup, including discovery of virtqueues
    // for the device, optional per-bus setup, reading and possibly writing
    // the device’s virtio configuration space, and population of virtqueues.
    // TODO(aeryz): still dunno how to do this
    device_init_fn();

    // 8. Set the DRIVER_OK status bit. At this point the device is “live”.
    write32(device_base, RegisterOffset::Status, Status::DriverOk);
    Ok(())
}

pub fn read32(device_base: usize, offset: RegisterOffset) -> u32 {
    unsafe { core::ptr::read_volatile((device_base + offset as usize) as *const u32) }
}

pub fn write32<V: Into<u32>>(device_base: usize, offset: RegisterOffset, value: V) {
    unsafe {
        core::ptr::write_volatile((device_base + offset as usize) as *mut u32, value.into());
    }
}
