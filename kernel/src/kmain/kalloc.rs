// Physical memory allocator, for user processes,
// kernel stacks, page-table pages,
// and pipe buffers. Allocates whole 4096-byte pages.

use super::spinlock::SpinLock;
use crate::memlayout::PHYSTOP;
use crate::riscv::{PGSIZE, pgroundup};
use core::cell::UnsafeCell;

unsafe extern "C" {
    // first address after kernel.
    // defined by kernel.ld.
    static end: [u8; 0];
}

#[inline(always)]
fn end_addr() -> usize {
    unsafe { end.as_ptr() as usize }
}

struct Run {
    next: *mut Run,
}

struct Kmem {
    lock: SpinLock,
    freelist: UnsafeCell<*mut Run>,
}

// Kmem is thread-safe because we use a lock.
unsafe impl Sync for Kmem {}

static KMEM: Kmem = Kmem {
    lock: SpinLock::initlock("kmem"),
    freelist: UnsafeCell::new(core::ptr::null_mut()),
};

pub fn kinit() {
    freerange(end_addr());
}

// Allocate one 4096-byte page of physical memory.
// Returns a pointer that the kernel can use.
// Returns 0 if the memory cannot be allocated.
pub fn kalloc() -> *mut u8 {
    let r;
    unsafe {
        KMEM.lock.acquire();
        let mut_freelist = KMEM.freelist.get();
        r = *mut_freelist;
        if !r.is_null() {
            *mut_freelist = (*r).next;
        }
        KMEM.lock.release();
    }

    let ptr = r as *mut u8;
    if !ptr.is_null() {
        unsafe {
            // Fill with junk.
            core::ptr::write_bytes(ptr, 1, PGSIZE);
        }
    }
    ptr
}

fn freerange(pa_start: usize) {
    let pa_start = pgroundup(pa_start);
    let pa_end = PHYSTOP - PGSIZE;
    for pa in (pa_start..=pa_end).step_by(PGSIZE) {
        unsafe {
            kfree(pa);
        }
    }
}

// Free the page of physical memory pointed at by pa,
// which normally should have been returned by a
// call to kalloc().  (The exception is when
// initializing the allocator; see kinit above.)
unsafe fn kfree(pa: usize) {
    if pa % PGSIZE != 0 || pa < end_addr() || pa >= PHYSTOP {
        panic!("kfree");
    }

    unsafe {
        // Fill with junk to catch dangling refs.
        core::ptr::write_bytes(pa as *mut u8, 1, PGSIZE);
        let r = pa as *mut Run;
        KMEM.lock.acquire();
        let mut_freelist = KMEM.freelist.get();
        (*r).next = *mut_freelist;
        *mut_freelist = r;
        KMEM.lock.release();
    }
}
