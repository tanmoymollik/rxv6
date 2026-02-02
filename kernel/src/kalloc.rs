use crate::arch::{self, Arch, CurrentArch};
use crate::memlayout;
use crate::spinlock::Spinlock;
use core::cell::Cell;

/// A physical page of `CurrentArch::page_size()` bytes.
pub struct PhysPage {
    pa: *mut u8,
}

impl PhysPage {
    /// Returns the base address of the page.
    pub fn get_addr(&self) -> usize {
        arch::ptr_address(self.pa)
    }
}

/// Allocate one page of physical memory.
/// Returns `Some(PhysPage)` if memory is available, otherwise return None.
pub fn kalloc() -> Option<PhysPage> {
    let r = KMEM.lock.with_lock(|| {
        let r = KMEM.free_list.get();
        if !r.is_null() {
            let next = unsafe { *(r as *mut *mut u8) };
            KMEM.free_list.set(next);
        }
        r
    });
    if r.is_null() {
        None
    } else {
        Some(PhysPage { pa: r })
    }
}

/// Create a linked list of free physical pages.
pub fn kinit() {
    let pa_start = arch::pg_round_up(end_addr());
    let pa_end = memlayout::PHYSTOP - CurrentArch::page_size();
    for pa in (pa_start..=pa_end).step_by(CurrentArch::page_size()) {
        kfree(pa as *mut u8);
    }
}

impl Drop for PhysPage {
    fn drop(&mut self) {
        kfree(self.pa);
    }
}

struct Kmem {
    lock: Spinlock,
    free_list: Cell<*mut u8>,
}

unsafe impl Sync for Kmem {}

static KMEM: Kmem = Kmem {
    lock: Spinlock::new("kalloc"),
    free_list: Cell::new(core::ptr::null_mut()),
};

unsafe extern "C" {
    // First address after kernel. Defined by kernel.ld.
    static end: [u8; 0];
}

#[inline(always)]
fn end_addr() -> usize {
    unsafe { arch::ptr_address(end.as_ptr()) }
}

// Free the page of physical memory pointed at by pa, which normally should
// have been returned by a call to kalloc(). (The exception is when
// initializing the allocator; see `kinit()` above.)
fn kfree(pa: *mut u8) {
    let pa = arch::ptr_address(pa);
    if pa % CurrentArch::page_size() != 0 || pa < end_addr() || pa >= memlayout::PHYSTOP {
        panic!("kfree: 0x{:x}", pa);
    }

    let pa = pa as *mut u8;
    unsafe {
        pa.write_bytes(1, CurrentArch::page_size());
    }
    KMEM.lock.with_lock(|| unsafe {
        (pa as *mut *mut u8).write(KMEM.free_list.get());
        KMEM.free_list.set(pa);
    });
}
