use crate::arch::{Arch, CurrentArch};
use crate::kutils;
use core::cell::Cell;
use core::hint::spin_loop;
use core::sync::atomic::{AtomicBool, Ordering};

pub struct SpinlockToken {
    _private: (),
}

/// Mutual exclusion lock.
/// Current implementation assumes that the lock will be created as a static
/// variable.
pub struct Spinlock {
    // Is the lock held?
    locked: AtomicBool,
    // For debugging:
    // Owner of the lock.
    name: &'static str,
    // The cpuid holding the lock.
    cpuid: Cell<Option<usize>>,
}

unsafe impl Sync for Spinlock {}

impl Spinlock {
    pub const fn new(name: &'static str) -> Self {
        Spinlock {
            locked: AtomicBool::new(false),
            name,
            cpuid: Cell::new(None),
        }
    }

    /// Runs the given closure with interrupts disabled.
    pub fn with_lock<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let token = self.acquire();
        let r = f();
        self.release(token);
        r
    }

    // Acquire the lock.
    // Loops (spins) until the lock is acquired.
    fn acquire(&self) -> SpinlockToken {
        // Disable interrupts to avoid deadlock.
        kutils::push_intr_off();
        if self.holding() {
            panic!("cpuid: {} acquire {}", self.cpuid.get().unwrap(), self.name);
        }

        // Use atomic swap instruction.
        while self.locked.swap(true, Ordering::Acquire) {
            spin_loop();
        }

        self.cpuid.set(Some(CurrentArch::cpuid()));
        SpinlockToken { _private: () }
    }

    // Release the lock.
    fn release(&self, _: SpinlockToken) {
        self.cpuid.set(None);

        // Use atomic assignment instruction.
        self.locked.store(false, Ordering::Release);
        // Enable interrupts again.
        kutils::pop_intr_off();
    }

    // Used for debugging. Will cause a panic if this cpu is holding and tries
    // to acquire again.
    fn holding(&self) -> bool {
        self.locked.load(Ordering::Relaxed) && self.cpuid.get().is_some()
    }
}
