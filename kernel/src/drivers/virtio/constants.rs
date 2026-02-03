/// Virtio device definitions for both the mmio interface and virtio
/// descriptors. Only tested with qemu.
///
/// The virtio spec:
/// https://docs.oasis-open.org/virtio/virtio/v1.1/virtio-v1.1.pdf

/// Virtio mmio control registers, mapped starting at 0x10001000, from qemu's
/// virtio_mmio.h
/// Magic number - 0x74726976
pub const VIRTIO_MMIO_MAGIC_VALUE: usize = 0x000;
/// Version: should be 2.
pub const VIRTIO_MMIO_VERSION: usize = 0x004;
/// Device type; 1 is net, 2 is disk.
pub const VIRTIO_MMIO_DEVICE_ID: usize = 0x008;
/// Should be 0x554d4551.
pub const VIRTIO_MMIO_VENDOR_ID: usize = 0x00c;
pub const VIRTIO_MMIO_DEVICE_FEATURES: usize = 0x010;
pub const VIRTIO_MMIO_DRIVER_FEATURES: usize = 0x020;
/// Select queue, write-only.
pub const VIRTIO_MMIO_QUEUE_SEL: usize = 0x030;
/// Max size of current queue, read-only.
pub const VIRTIO_MMIO_QUEUE_NUM_MAX: usize = 0x034;
/// Size of current queue, write-only.
pub const VIRTIO_MMIO_QUEUE_NUM: usize = 0x038;
/// Ready bit.
pub const VIRTIO_MMIO_QUEUE_READY: usize = 0x044;
pub const VIRTIO_MMIO_QUEUE_NOTIFY: usize = 0x050; // write-only
pub const VIRTIO_MMIO_INTERRUPT_STATUS: usize = 0x060; // read-only
pub const VIRTIO_MMIO_INTERRUPT_ACK: usize = 0x064; // write-only
pub const VIRTIO_MMIO_STATUS: usize = 0x070; // read/write
/// Physical address for descriptor table divided into two 32-bit registers,
/// write-only.
pub const VIRTIO_MMIO_QUEUE_DESC_LOW: usize = 0x080;
pub const VIRTIO_MMIO_QUEUE_DESC_HIGH: usize = 0x084;
/// Physical address for available ring divided into two 32-bit registers,
/// write-only.
pub const VIRTIO_MMIO_DRIVER_DESC_LOW: usize = 0x090;
pub const VIRTIO_MMIO_DRIVER_DESC_HIGH: usize = 0x094;
/// Physical address for used ring divided into two 32-bit registers,
/// write-only.
pub const VIRTIO_MMIO_DEVICE_DESC_LOW: usize = 0x0a0;
pub const VIRTIO_MMIO_DEVICE_DESC_HIGH: usize = 0x0a4;

/// Status register bits, from qemu's virtio_config.h
pub const VIRTIO_CONFIG_S_ACKNOWLEDGE: u32 = 1;
pub const VIRTIO_CONFIG_S_DRIVER: u32 = 2;
pub const VIRTIO_CONFIG_S_DRIVER_OK: u32 = 4;
pub const VIRTIO_CONFIG_S_FEATURES_OK: u32 = 8;

/// Device feature bits.
/// Disk is read-only.
pub const VIRTIO_BLK_F_RO: usize = 5;
/// Supports scsi command passthrough.
pub const VIRTIO_BLK_F_SCSI: usize = 7;
/// Writeback mode available in config.
pub const VIRTIO_BLK_F_CONFIG_WCE: usize = 11;
/// Support more than one vq.
pub const VIRTIO_BLK_F_MQ: usize = 12;
pub const VIRTIO_F_ANY_LAYOUT: usize = 27;
pub const VIRTIO_RING_F_INDIRECT_DESC: usize = 28;
pub const VIRTIO_RING_F_EVENT_IDX: usize = 29;

/// This many virtio descriptors, must be a power of two.
/// This must also fit into a `u16` as that is the maximum index allowed in the
/// descriptor table.
pub const NUM: usize = 8;
/// Virtio sector size.
pub const SSIZE: usize = 512;

/// A single descriptor, from virtio spec.
#[repr(C)]
pub struct VirtqDesc {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}
/// Chained with another descriptor.
pub const VRING_DESC_F_NEXT: u16 = 1;
/// Device writes (vs read).
pub const VRING_DESC_F_WRITE: u16 = 2;

/// the (entire) avail ring, from the spec.
#[repr(C, packed)]
pub struct VirtqAvail {
    /// Always zero.
    pub flags: u16,
    /// Driver will write ring[idx] next.
    pub idx: u16,
    /// Descriptor numbers of chain heads.
    pub ring: [u16; NUM],
    pub unused: u16,
}

/// One entry in the "used" ring, with which the device tells the driver about
/// completed requests.
#[repr(C)]
pub struct VirtqUsedElem {
    /// Index of start of completed descriptor chain.
    pub id: u32,
    pub len: u32,
}

#[repr(C)]
pub struct VirtqUsed {
    /// Always zero.
    pub flags: u16,
    /// Device increments when it adds a ring[] entry.
    pub idx: u16,
    pub ring: [VirtqUsedElem; NUM],
}

/// These are specific to virtio block devices, e.g. disks, described in Section
/// 5.2 of the spec.
/// Read the disk.
pub const VIRTIO_BLK_T_IN: u32 = 0;
/// Write the disk.
pub const VIRTIO_BLK_T_OUT: u32 = 1;

/// The format of the first descriptor in a disk request. To be followed by two
/// more descriptors containing the block, and a one-byte status.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VirtioBlkReq {
    /// VIRTIO_BLK_T_IN or ..._OUT
    pub r#type: u32,
    pub reserved: u32,
    pub sector: u64,
}

impl VirtioBlkReq {
    pub const fn new() -> Self {
        VirtioBlkReq {
            r#type: 0,
            reserved: 0,
            sector: 0,
        }
    }
}
