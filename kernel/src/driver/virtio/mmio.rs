#![allow(unused)]

const VIRTIO_MMIO_MAGIC: u32 = 0x7472_6976;
const VIRTIO_MMIO_VERSION: u32 = 0x2;

#[repr(usize)]
enum RegisterOffset {
    /// 0x74726976 (a Little Endian equivalent of the “virt” string).
    Magic = 0x0,
    /// 0x2. Note: Legacy devices (see 4.2.4 Legacy interface) used 0x1.
    Version = 0x4,
    /// See 5 Device Types for possible values. Value zero (0x0) is used
    /// to define a system memory map with placeholder devices at static,
    /// well known addresses, assigning functions to them depending on user’s needs.
    DeviceId = 0x8,
    /// Virtio Subsystem Vendor ID
    VendorId = 0xc,
    /// Flags representing features the device supports
    /// Reading from this register returns 32 consecutive flag bits,
    /// the least significant bit depending on the last value written
    /// to DeviceFeaturesSel. Access to this register returns bits
    /// DeviceFeaturesSel ∗ 32 to (DeviceFeaturesSel ∗ 32) + 31, eg.
    /// feature bits 0 to 31 if DeviceFeaturesSel is set to 0 and features
    /// bits 32 to 63 if DeviceFeaturesSel is set to 1. Also see 2.2 Feature Bits.
    DeviceFeatures = 0x10,
    /// Device (host) features word selection.
    /// Writing to this register selects a set of 32 device feature bits accessible
    /// by reading from DeviceFeatures.
    DeviceFeaturesSel = 0x14,
    /// Flags representing device features understood and activated by the driver
    /// Writing to this register sets 32 consecutive flag bits, the least significant
    /// bit depending on the last value written to DriverFeaturesSel. Access to this
    /// register sets bits DriverFeaturesSel ∗ 32 to (DriverFeaturesSel ∗ 32) + 31,
    /// eg. feature bits 0 to 31 if DriverFeaturesSel is set to 0 and features bits 32
    /// to 63 if DriverFeaturesSel is set to 1. Also see 2.2 Feature Bits.
    DriverFeatures = 0x20,
    /// Activated (guest) features word selection
    /// Writing to this register selects a set of 32 activated feature bits accessible
    /// by writing to DriverFeatures.
    DriverFeaturesSel = 0x24,
    /// Virtqueue index
    /// Writing to this register selects the virtqueue that the following operations on
    /// QueueSizeMax, QueueSize, QueueReady, QueueDescLow, QueueDescHigh, QueueDriverlLow,
    /// QueueDriverHigh, QueueDeviceLow, QueueDeviceHigh and QueueReset apply to.
    QueueSel = 0x30,
    /// Maximum virtqueue size
    /// Reading from the register returns the maximum size (number of elements) of the
    /// queue the device is ready to process or zero (0x0) if the queue is not available.
    /// This applies to the queue selected by writing to QueueSel. Note: QueueSizeMax was
    /// previously known as QueueNumMax.
    QueueSizeMax = 0x34,
    QueueSize = 0x38,
    QueueReady = 0x44,
    QueueNotify = 0x50,
    InterruptStatus = 0x60,
    InterruptAck = 0x64,
    Status = 0x70,
    QueueDescLow = 0x80,
    QueueDescHigh = 0x84,
    QueueDriverLow = 0x090,
    QueueDriverHigh = 0x094,
    QueueDeviceLow = 0x0a0,
    QueueDeviceHigh = 0x0a4,
    SHMSel = 0x0ac,
    SHMLenLow = 0x0b0,
    SHMLenHigh = 0x0b4,
    SHMBaseLow = 0x0b8,
    SHMBaseHigh = 0x0bc,
    QueueReset = 0x0c0,
    ConfigGeneration = 0x0fc,
    Config = 0x100,
}
