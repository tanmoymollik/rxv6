use super::proc::{Cpu, mycpu};
use crate::riscv::{intr_get, intr_off, intr_on};
use core::cell::UnsafeCell;
use core::hint::spin_loop;
use core::sync::atomic::{AtomicBool, Ordering, fence};

// Mutual exclusion lock.
pub struct SpinLock {
    // Is the lock held?
    locked: AtomicBool,
    // For debugging:
    name: &'static str,
    // The cpu holding the lock.
    cpu: UnsafeCell<Option<&'static Cpu>>,
}

unsafe impl Sync for SpinLock {}

impl SpinLock {
    pub const fn initlock(name: &'static str) -> Self {
        SpinLock {
            locked: AtomicBool::new(false),
            name,
            cpu: UnsafeCell::new(None),
        }
    }

    // Acquire the lock.
    // Loops (spins) until the lock is acquired.
    pub fn acquire(&self) {
        // disable interrupts to avoid deadlock.
        Self::push_off();
        if self.holding() {
            panic!("acquire");
        }

        // Use atomic swap instruction.
        while self.locked.swap(true, Ordering::Acquire) {
            spin_loop();
        }

        // Tell the rust compiler and the processor to not move loads or stores
        // past this point, to ensure that the critical section's memory
        // references happen strictly after the lock is acquired.
        fence(Ordering::SeqCst);
        unsafe {
            *self.cpu.get() = Some(mycpu());
        }
    }

    // Release the lock.
    pub fn release(&self) {
        if !self.holding() {
            panic!("release");
        }
        unsafe {
            *self.cpu.get() = None;
        }

        // Tell the rust compiler and the CPU to not move loads or stores
        // past this point, to ensure that all the stores in the critical
        // section are visible to other CPUs before the lock is released,
        // and that loads in the critical section occur strictly before
        // the lock is released.
        fence(Ordering::SeqCst);
        // Use atomic assignment instruction.
        self.locked.store(false, Ordering::Release);
        Self::pop_off();
    }

    fn holding(&self) -> bool {
        let cpu = unsafe { *self.cpu.get() };
        self.locked.load(Ordering::Relaxed) && cpu.is_some_and(|c| core::ptr::eq(c, mycpu()))
    }

    // push_off/pop_off are like intr_off()/intr_on() except that they are matched:
    // it takes two pop_off()s to undo two push_off()s.  Also, if interrupts
    // are initially off, then push_off, pop_off leaves them off.

    pub fn push_off() {
        let old = intr_get();
        // disable interrupts to prevent an involuntary context
        // switch while using mycpu().
        intr_off();

        let cpu = mycpu();
        if cpu.noff() == 0 {
            cpu.set_intena(old);
        }
        cpu.add_noff(1);
    }

    pub fn pop_off() {
        let cpu = mycpu();
        if intr_get() {
            panic!("pop_off - interruptible");
        }
        if cpu.noff() < 1 {
            panic!("pop_off");
        }
        cpu.sub_noff(1);
        if cpu.noff() == 0 && cpu.intena() {
            intr_on();
        }
    }
}
