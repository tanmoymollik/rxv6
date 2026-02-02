use core::sync::atomic::{AtomicBool, Ordering};

pub struct KStateToken {
    _private: (),
}

static PANICKING: AtomicBool = AtomicBool::new(false);
static PANICKED: AtomicBool = AtomicBool::new(false);

/// Returns true if the kernel is currently panicking.
/// Doesn't require a lock because PANICKING is only ever set to true
/// once, and never set back to false.
/// Also if the current cpu is panicking, we don't need other cpus to panic
/// immediately. So synchonization is not required.
pub fn is_panicking() -> bool {
    PANICKING.load(Ordering::SeqCst)
}

/// Sets the panicked state to true.
pub fn set_panicking() {
    PANICKING.store(true, Ordering::SeqCst);
}

/// Returns true if the kernel has panicked.
/// Doesn't require a lock because PANICKED is only ever set to true
/// once, and never set back to false.
/// Also if the current cpu has panicked, we don't need other cpus to panic
/// immediately. So synchonization is not required.
pub fn has_panicked() -> bool {
    PANICKED.load(Ordering::SeqCst)
}

/// Sets the panicked state to true.
pub fn set_panicked() {
    PANICKED.store(true, Ordering::SeqCst);
}

pub fn read_kstate<F, R>(f: F) -> R
where
    for<'a> F: FnOnce(&'a KStateToken) -> R,
{
    // Acquire spinlock.
    let token = KStateToken { _private: () };
    let r = f(&token);
    // Release spinlock.
    r
}

pub fn update_kstate<F, R>(f: F) -> R
where
    for<'a> F: FnOnce(&'a KStateToken) -> R,
{
    // Acquire spinlock.
    let token = KStateToken { _private: () };
    let r = f(&token);
    // Release spinlock.
    r
}
