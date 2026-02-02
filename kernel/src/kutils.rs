use crate::arch::{Arch, CurrentArch};
use crate::cpu;

/// Runs the given closure with interrupts disabled.
pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    push_intr_off();
    let r = f();
    pop_intr_off();
    r
}

/// Disables interrupts and increments the nesting interrupt off count of the
/// current CPU. It also records whether interrupts were enabled before the call.
/// This must be matched by a corresponding call to pop_intr_off.
pub fn push_intr_off() {
    let old = CurrentArch::interrupts_enabled();
    // Disable interrupts to prevent an involuntary context switch while using mycpu().
    CurrentArch::disable_interrupts();
    let c = cpu::mycpu();
    if c.noff == 0 {
        c.intena = old;
    }
    c.noff += 1;
}

/// Decrements the nesting interrupt off count of the current CPU. If the count
/// reaches zero and interrupts were enabled before the corresponding
/// push_intr_off, it re-enables interrupts.
/// This must be matched by a prior call to push_intr_off.
pub fn pop_intr_off() {
    let c = cpu::mycpu();
    if CurrentArch::interrupts_enabled() {
        panic!("pop_intr_off - interruptible");
    }
    if c.noff < 1 {
        panic!("pop_intr_off");
    }
    c.noff -= 1;
    if c.noff == 0 && c.intena {
        CurrentArch::enable_interrupts();
    }
}
