use crate::arch::{self, Arch, CurrentArch};
use crate::buf::Buf;
use crate::channel::Channel;
use crate::kalloc::{PhysPage, kalloc};
use crate::memlayout;
use crate::proc::{sleep, wakeup};
use crate::spinlock::{Spinlock, SpinlockToken};
use core::marker::PhantomData;
use core::mem::size_of;
use core::sync::atomic::Ordering;
use core::sync::atomic::fence;
use kernelapi::fs::BSIZE;

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
        let page = alloc();
        let addr = page.get_addr();
        write_reg(VIRTIO_MMIO_QUEUE_DESC_LOW, addr as u32);
        write_reg(VIRTIO_MMIO_QUEUE_DESC_HIGH, (addr >> 32) as u32);
        DISK.desc = VSList::new(page);

        let page = alloc();
        let addr = page.get_addr();
        write_reg(VIRTIO_MMIO_DRIVER_DESC_LOW, addr as u32);
        write_reg(VIRTIO_MMIO_DRIVER_DESC_HIGH, (addr >> 32) as u32);
        DISK.avail = VSList::new(page);

        let page = alloc();
        let addr = page.get_addr();
        write_reg(VIRTIO_MMIO_DEVICE_DESC_LOW, addr as u32);
        write_reg(VIRTIO_MMIO_DEVICE_DESC_HIGH, (addr >> 32) as u32);
        DISK.used = VSList::new(alloc());

        // All NUM descriptors start out unused.
        for i in 0..NUM {
            DISK.free[i] = true;
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
// This must also fit into a `u16` as that is the maximum index allowed in the
// descriptor table.
const NUM: usize = 8;
// Virtio sector size.
const SSIZE: usize = 512;

// A single descriptor, from virtio spec.
#[repr(C)]
struct VirtqDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}
// Chained with another descriptor.
const VRING_DESC_F_NEXT: u16 = 1;
// Device writes (vs read).
const VRING_DESC_F_WRITE: u16 = 2;
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
#[repr(C)]
struct VirtqUsedElem {
    // Index of start of completed descriptor chain.
    id: u32,
    len: u32,
}

#[repr(C)]
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
const VIRTIO_BLK_T_IN: u32 = 0;
// Write the disk.
const VIRTIO_BLK_T_OUT: u32 = 1;

// The format of the first descriptor in a disk request. To be followed by two
// more descriptors containing the block, and a one-byte status.
#[repr(C)]
#[derive(Clone, Copy)]
struct VirtioBlkReq {
    // VIRTIO_BLK_T_IN or ..._OUT
    r#type: u32,
    reserved: u32,
    sector: u64,
}

impl VirtioBlkReq {
    const fn new() -> Self {
        VirtioBlkReq {
            r#type: 0,
            reserved: 0,
            sector: 0,
        }
    }
}

// A wrapper class that owns a page worth of memory and provides a functional
// interface of a list (or ring) of virtq device structs.
struct VSList<T: Sized> {
    page: Option<PhysPage>,
    _ph: PhantomData<T>,
}

impl<T> VSList<T> {
    const fn uninit() -> Self {
        VSList {
            page: None,
            _ph: PhantomData,
        }
    }

    fn new(page: PhysPage) -> Self {
        unsafe {
            page.get_ptr().write_bytes(0, CurrentArch::page_size());
        }
        VSList {
            page: Some(page),
            _ph: PhantomData,
        }
    }

    fn get_page(&self) -> &PhysPage {
        if self.page.is_none() {
            panic!("virtio_disk - tried to access before init");
        }
        self.page.as_ref().unwrap()
    }

    fn read(&self, ind: usize) -> &T {
        let s_size = size_of::<T>();
        let s_ptr = (self.get_page().get_addr() + s_size * ind) as *const T;
        unsafe { &*s_ptr }
    }

    fn write(&self, ind: usize, s: T) {
        let s_size = size_of::<T>();
        let s_ptr = (self.get_page().get_addr() + s_size * ind) as *mut T;
        unsafe {
            s_ptr.write_volatile(s);
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct DiskInfo {
    buf: *const Buf,
    status: u8,
}

struct Disk {
    // A set (not a ring) of DMA descriptors, with which the driver tells the
    // device where to read and write individual disk operations. There are
    // `NUM` descriptors. Most commands consist of a "chain" (a linked list) of
    // a coupe of these descriptors.
    desc: VSList<VirtqDesc>,
    // A ring in which the driver writes descriptor numbers that the driver
    // would like the device to process. It only includes the head descriptor
    // of each chain. The ring has `NUM` elements.
    avail: VSList<VirtqAvail>,
    // A ring in which the device writes descriptor numbers that the device has
    // finished processing (just the head of each chain). There are `NUM` used
    // ring entries.
    used: VSList<VirtqUsed>,

    // Our own book-keeping.
    // Is a descriptor free?
    free: [bool; NUM],
    // We've looked this far in `used[2..NUM]`.
    used_idx: u16,
    // Track info about in-flight operations, for use when completion interrupt
    // arrives. Indexed by first descriptor index of chain.
    info: [DiskInfo; NUM],
    // Disk command headers. One-for-one with descriptors, for convenience.
    ops: [VirtioBlkReq; NUM],
    // Spinlock to guard the disk.
    lock: Spinlock,
}

unsafe impl Sync for Disk {}

// desc, avail and used is initialized in `virtio_disk_init()`.
static mut DISK: Disk = Disk {
    desc: VSList::uninit(),
    avail: VSList::uninit(),
    used: VSList::uninit(),
    free: [false; NUM],
    used_idx: 0,
    info: [DiskInfo {
        buf: core::ptr::null(),
        status: 0,
    }; NUM],
    ops: [VirtioBlkReq::new(); NUM],
    lock: Spinlock::new("vdisk_lock"),
};

impl Disk {
    // Find a free descriptor, mark it non-free, return its index.
    // Returns `None` if there are no free descriptors.
    fn alloc_desc(&mut self, _: &SpinlockToken) -> Option<u16> {
        for i in 0..NUM {
            if self.free[i] {
                self.free[i] = false;
                return Some(i as u16);
            }
        }
        None
    }

    // Mark a descriptor as free.
    fn free_desc(&mut self, idx: u16, _: &SpinlockToken) {
        let idx = idx as usize;
        if idx >= NUM {
            panic!("free_desc 1");
        }
        if self.free[idx] {
            panic!("free_desc 2");
        }

        let desc = VirtqDesc {
            addr: 0,
            len: 0,
            flags: 0,
            next: 0,
        };
        self.desc.write(idx, desc);
        self.free[idx] = true;
        wakeup(Channel::VirtioDescFree);
    }

    // Free a chain of descriptors.
    fn free_chain(&mut self, mut idx: u16, tk: &SpinlockToken) {
        loop {
            let tmp_desc = self.desc.read(idx as usize);
            let flag = tmp_desc.flags;
            let nxt = tmp_desc.next;
            self.free_desc(idx, tk);
            if flag & (VRING_DESC_F_NEXT as u16) != 0 {
                idx = nxt;
            } else {
                break;
            }
        }
    }

    // Allocate three descriptors (they need not be contigous). Disk transfers
    // always use three descriptors.
    fn alloc3_desc(&mut self, tk: &SpinlockToken) -> Option<[u16; 3]> {
        let mut idx = [0u16; 3];
        for i in 0..3 {
            match self.alloc_desc(tk) {
                Some(alc_id) => idx[i] = alc_id,
                None => {
                    // Couldn't allocate a descriptor. Free all (if any)
                    // previously allocated descriptors.
                    for j in 0..i {
                        self.free_desc(idx[j], tk);
                    }
                    return None;
                }
            }
        }
        Some(idx)
    }
}

fn virtio_disk_rw(buf: *mut Buf, write: bool) {
    let sector = unsafe { ((*buf).blockno as usize * (BSIZE / SSIZE)) as u64 };
    let disk = &raw mut DISK;
    let tk = unsafe { (*disk).lock.acquire() };

    // The virtio spec's Section 5.2 says that legacy block operations use
    // three descriptors: one for type/reserved/sector, one for the data,
    // one for a 1-byte status result.

    // Allocate the three descriptors.
    let idx: [u16; 3];
    loop {
        match unsafe { (*disk).alloc3_desc(&tk) } {
            Some(_idx) => {
                idx = _idx;
                break;
            }
            None => sleep(Channel::VirtioDescFree),
        }
    }

    // Format the three descriptors. Qemu's virtio-blk.c reads them.
    let buf0_addr = unsafe {
        let buf0 = &mut (*disk).ops[idx[0] as usize];
        if write {
            buf0.r#type = VIRTIO_BLK_T_OUT;
        } else {
            buf0.r#type = VIRTIO_BLK_T_IN;
        }
        buf0.reserved = 0;
        buf0.sector = sector;
        arch::ptr_address(buf0 as *const VirtioBlkReq)
    };

    let desc = VirtqDesc {
        addr: buf0_addr as u64,
        len: size_of::<VirtioBlkReq>() as u32,
        flags: VRING_DESC_F_NEXT,
        next: idx[1],
    };
    unsafe {
        (*disk).desc.write(idx[0] as usize, desc);
    }

    let desc = VirtqDesc {
        addr: arch::ptr_address(unsafe { &(*buf).data }) as u64,
        len: BSIZE as u32,
        flags: if write {
            // Device reads from buf.data.
            0
        } else {
            // Device writes to buf.data.
            VRING_DESC_F_WRITE
        },
        next: idx[2],
    };
    unsafe {
        (*disk).desc.write(idx[1] as usize, desc);
    }

    let status_addr = unsafe {
        let status = &mut (*disk).info[idx[0] as usize].status;
        *status = 0xff;
        arch::ptr_address(status)
    };
    let desc = VirtqDesc {
        addr: status_addr as u64,
        len: 1,
        // Device writes the status.
        flags: VRING_DESC_F_WRITE,
        next: 0,
    };
    unsafe {
        (*disk).desc.write(idx[2] as usize, desc);
    }

    // Record the struct buf for virtio_disk_intr().
    unsafe {
        (*buf).disk = true;
        (*disk).info[idx[0] as usize].buf = buf;
    }

    // Tell the device the first index in our chain of descriptors.
    unsafe {
        let _idx = (*disk).avail.read(0).idx % (NUM as u16);
        // (*disk).avail.ring[_idx] = idx[0];
    }

    fence(Ordering::SeqCst);
    // Tell the device another avail ring entry is available.
    unsafe {
        //(*disk).avail.idx += 1;
    }
    fence(Ordering::SeqCst);

    // Value is the queue number.
    write_reg(VIRTIO_MMIO_QUEUE_NOTIFY, 0);

    // Wait for virtio_disk_intr() to say request has finished.
    while unsafe { (*buf).disk } {
        sleep(Channel::VirtioReqFinished);
    }

    // Cleanup.
    unsafe {
        (*disk).info[idx[0] as usize].buf = core::ptr::null();
        (*disk).free_chain(idx[0], &tk);
        (*disk).lock.release(tk);
    }
}

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
