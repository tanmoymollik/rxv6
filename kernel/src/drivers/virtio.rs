use crate::arch::{Arch, CurrentArch};
use crate::kalloc::{PhysPage, kalloc};
use crate::memlayout;

/// Virtio device definitions for both the mmio interface and virtio
/// descriptors. Only tested with qemu.
///
/// The virtio spec:
/// https://docs.oasis-open.org/virtio/virtio/v1.1/virtio-v1.1.pdf

pub fn virtio_disk_init() {
    if read_reg(VIRTIO_MMIO_MAGIC_VALUE) != 0x74726976
        || read_reg(VIRTIO_MMIO_VERSION) != 2
        || read_reg(VIRTIO_MMIO_DEVICE_ID) != 2
        || read_reg(VIRTIO_MMIO_VENDOR_ID) != 0x554d4551
    {
        panic!("could not find virtio disk");
    }

    let mut status = 0u32;
    // Reset device.
    write_reg(VIRTIO_MMIO_STATUS, status);

    // Set ACKNOWLEDGE status bit.
    status |= VIRTIO_CONFIG_S_ACKNOWLEDGE;
    write_reg(VIRTIO_MMIO_STATUS, status);

    // Set DRIVER status bit.
    status |= VIRTIO_CONFIG_S_DRIVER;
    write_reg(VIRTIO_MMIO_STATUS, status);

    // Negotiate features.
    let mut features = read_reg(VIRTIO_MMIO_DEVICE_FEATURES);
    features &= !(1 << VIRTIO_BLK_F_RO);
    features &= !(1 << VIRTIO_BLK_F_SCSI);
    features &= !(1 << VIRTIO_BLK_F_CONFIG_WCE);
    features &= !(1 << VIRTIO_BLK_F_MQ);
    features &= !(1 << VIRTIO_F_ANY_LAYOUT);
    features &= !(1 << VIRTIO_RING_F_EVENT_IDX);
    features &= !(1 << VIRTIO_RING_F_INDIRECT_DESC);
    write_reg(VIRTIO_MMIO_DRIVER_FEATURES, features);

    // Tell device that feature negotiation is complete.
    status |= VIRTIO_CONFIG_S_FEATURES_OK;
    write_reg(VIRTIO_MMIO_STATUS, status);

    // Re-read status to ensure FEATURES_OK is set.
    status = read_reg(VIRTIO_MMIO_STATUS);
    if (status & VIRTIO_CONFIG_S_FEATURES_OK) == 0 {
        panic!("virtio disk FEATURES_OK unset");
    }

    // Initialize queue 0.
    write_reg(VIRTIO_MMIO_QUEUE_SEL, 0);

    // Ensure queue 0 is not in use.
    if read_reg(VIRTIO_MMIO_QUEUE_READY) != 0 {
        panic!("virtio disk should not be ready");
    }

    // Check maximum queue size.
    let max = read_reg(VIRTIO_MMIO_QUEUE_NUM_MAX);
    if max == 0 {
        panic!("virtio disk has no queue 0");
    }
    if (max as usize) < NUM {
        panic!("virtio disk max queue too short");
    }

    let alloc = || match kalloc() {
        Some(page) => page,
        None => panic!("virtio disk kalloc"),
    };

    // Write physical addresses.
    unsafe {
        // Allocate and zero queue memory.
        DISK.desc = VirtqStructList::new(alloc());
        DISK.avail = VirtqStructList::new(alloc());
        DISK.used = VirtqStructList::new(alloc());

        write_reg(VIRTIO_MMIO_QUEUE_DESC_LOW, DISK.desc.page.get_addr() as u32);
        write_reg(
            VIRTIO_MMIO_QUEUE_DESC_HIGH,
            (DISK.desc.page.get_addr() >> 32) as u32,
        );
        write_reg(
            VIRTIO_MMIO_DRIVER_DESC_LOW,
            DISK.avail.page.get_addr() as u32,
        );
        write_reg(
            VIRTIO_MMIO_DRIVER_DESC_HIGH,
            (DISK.avail.page.get_addr() >> 32) as u32,
        );
        write_reg(
            VIRTIO_MMIO_DEVICE_DESC_LOW,
            DISK.used.page.get_addr() as u32,
        );
        write_reg(
            VIRTIO_MMIO_DEVICE_DESC_HIGH,
            (DISK.used.page.get_addr() >> 32) as u32,
        );

        // All NUM descriptors start out unused.
        for i in 0..NUM {
            DISK.free[i] = 1;
        }
    }

    // Set queue size.
    write_reg(VIRTIO_MMIO_QUEUE_NUM, NUM as u32);
    // Queue is ready.
    write_reg(VIRTIO_MMIO_QUEUE_READY, 0x1);

    // Tell device we're completely ready.
    status |= VIRTIO_CONFIG_S_DRIVER_OK;
    write_reg(VIRTIO_MMIO_STATUS, status);

    // plic.rs and trap.rs arrange for interrupts from `VIRTIO0_IRQ`.
}

// Virtio mmio control registers, mapped starting at 0x10001000, from qemu's
// virtio_mmio.h
// Magic number - 0x74726976
const VIRTIO_MMIO_MAGIC_VALUE: usize = 0x000;
// Version: should be 2.
const VIRTIO_MMIO_VERSION: usize = 0x004;
// Device type; 1 is net, 2 is disk.
const VIRTIO_MMIO_DEVICE_ID: usize = 0x008;
// Should be 0x554d4551.
const VIRTIO_MMIO_VENDOR_ID: usize = 0x00c;
const VIRTIO_MMIO_DEVICE_FEATURES: usize = 0x010;
const VIRTIO_MMIO_DRIVER_FEATURES: usize = 0x020;
// Select queue, write-only.
const VIRTIO_MMIO_QUEUE_SEL: usize = 0x030;
// Max size of current queue, read-only.
const VIRTIO_MMIO_QUEUE_NUM_MAX: usize = 0x034;
// Size of current queue, write-only.
const VIRTIO_MMIO_QUEUE_NUM: usize = 0x038;
// Ready bit.
const VIRTIO_MMIO_QUEUE_READY: usize = 0x044;
const VIRTIO_MMIO_QUEUE_NOTIFY: usize = 0x050; // write-only
const VIRTIO_MMIO_INTERRUPT_STATUS: usize = 0x060; // read-only
const VIRTIO_MMIO_INTERRUPT_ACK: usize = 0x064; // write-only
const VIRTIO_MMIO_STATUS: usize = 0x070; // read/write
// Physical address for descriptor table divided into two 32-bit registers,
// write-only.
const VIRTIO_MMIO_QUEUE_DESC_LOW: usize = 0x080;
const VIRTIO_MMIO_QUEUE_DESC_HIGH: usize = 0x084;
// Physical address for available ring divided into two 32-bit registers,
// write-only.
const VIRTIO_MMIO_DRIVER_DESC_LOW: usize = 0x090;
const VIRTIO_MMIO_DRIVER_DESC_HIGH: usize = 0x094;
// Physical address for used ring divided into two 32-bit registers,
// write-only.
const VIRTIO_MMIO_DEVICE_DESC_LOW: usize = 0x0a0;
const VIRTIO_MMIO_DEVICE_DESC_HIGH: usize = 0x0a4;

// Status register bits, from qemu's virtio_config.h
const VIRTIO_CONFIG_S_ACKNOWLEDGE: u32 = 1;
const VIRTIO_CONFIG_S_DRIVER: u32 = 2;
const VIRTIO_CONFIG_S_DRIVER_OK: u32 = 4;
const VIRTIO_CONFIG_S_FEATURES_OK: u32 = 8;

// Device feature bits.
// Disk is read-only.
const VIRTIO_BLK_F_RO: usize = 5;
// Supports scsi command passthrough.
const VIRTIO_BLK_F_SCSI: usize = 7;
// Writeback mode available in config.
const VIRTIO_BLK_F_CONFIG_WCE: usize = 11;
// Support more than one vq.
const VIRTIO_BLK_F_MQ: usize = 12;
const VIRTIO_F_ANY_LAYOUT: usize = 27;
const VIRTIO_RING_F_INDIRECT_DESC: usize = 28;
const VIRTIO_RING_F_EVENT_IDX: usize = 29;

// This many virtio descriptors, must be a power of two.
const NUM: usize = 8;

// A single descriptor, from virtio spec.
#[repr(C, packed)]
struct VirtqDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}
// Chained with another descriptor.
const VRING_DESC_F_NEXT: usize = 1;
// Device writes (vs read).
const VRING_DESC_F_WRITE: usize = 2;
// the (entire) avail ring, from the spec.

#[repr(C, packed)]
struct VirtqAvail {
    // Always zero.
    flags: u16,
    // Driver will write ring[idx] next.
    idx: u16,
    // Descriptor numbers of chain heads.
    ring: [u16; NUM],
    unused: u16,
}

// One entry in the "used" ring, with which the device tells the driver about
// completed requests.
#[repr(C, packed)]
struct VirtqUsedElem {
    // Index of start of completed descriptor chain.
    id: u32,
    len: u32,
}

#[repr(C, packed)]
struct VirtqUsed {
    // Always zero.
    flags: u16,
    // Device increments when it adds a ring[] entry.
    idx: u16,
    ring: [VirtqUsedElem; NUM],
}

// These are specific to virtio block devices, e.g. disks, described in Section
// 5.2 of the spec.
// Read the disk.
const VIRTIO_BLK_T_IN: usize = 0;
// Write the disk.
const VIRTIO_BLK_T_OUT: usize = 1;

// The format of the first descriptor in a disk request. To be followed by two
// more descriptors containing the block, and a one-byte status.
struct VirtioBlkReq {
    // VIRTIO_BLK_T_IN or ..._OUT
    r#type: u32,
    reserved: u32,
    sector: u64,
}

struct VirtqStructList<T: Sized> {
    page: PhysPage,
    _ph: core::marker::PhantomData<T>,
}

impl<T> VirtqStructList<T> {
    fn new(page: PhysPage) -> Self {
        unsafe {
            (page.get_addr() as *mut u8).write_bytes(0, CurrentArch::page_size());
        }
        VirtqStructList {
            page,
            _ph: core::marker::PhantomData,
        }
    }

    fn write(&self, ind: usize, s: T) {
        let s_size = core::mem::size_of::<T>();
        let s_ptr = (self.page.get_addr() + s_size * ind) as *mut T;
        unsafe {
            s_ptr.write_volatile(s);
        }
    }
}

struct DiskInfo {
    status: u8,
}

struct Disk {
    // A set (not a ring) of DMA descriptors, with which the driver tells the
    // device where to read and write individual disk operations. There are
    // `NUM` descriptors. Most commands consist of a "chain" (a linked list) of
    // a coupe of these descriptors.
    desc: VirtqStructList<VirtqDesc>,
    // A ring in which the driver writes descriptor numbers that the driver
    // would like the device to process. It only includes the head descriptor
    // of each chain. The ring has `NUM` elements.
    avail: VirtqStructList<VirtqAvail>,
    // A ring in which the device writes descriptor numbers that the device has
    // finished processing (just the head of each chain). There are `NUM` used
    // ring entries.
    used: VirtqStructList<VirtqUsed>,
    // Our own book-keeping.
    // Is a descriptor free?
    free: [u8; NUM],
    // We've looked this far in `used[2..NUM]`.
    used_idx: u16,
    // Track info about in-flight operations, for use when completion interrupt
    // arrives. Indexed by first descriptor index of chain.
    info: [DiskInfo; NUM],
    // Disk command headers. One-for-one with descriptors, for convenience.
    ops: [VirtioBlkReq; NUM],
}

// Zero out all fields. The device only reads desc, avail and used entries
// which is initialized in `virtio_disk_init()`. The rest of the memebers can
// have 0 as a valid value and updated when used.
static mut DISK: Disk = unsafe { core::mem::MaybeUninit::zeroed().assume_init() };

// The VIRTIO control registers are memory-mapped at address
// `memlayout::VIRTIO0`. This macro returns the address of one of the registers.
#[inline(always)]
fn reg(reg: usize) -> usize {
    memlayout::VIRTIO0 + reg as usize
}

#[inline(always)]
fn read_reg(r: usize) -> u32 {
    unsafe { (reg(r) as *const u32).read_volatile() }
}

#[inline(always)]
fn write_reg(r: usize, v: u32) {
    unsafe {
        (reg(r) as *mut u32).write_volatile(v);
    }
}
