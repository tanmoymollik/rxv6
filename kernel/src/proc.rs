use crate::channel::Channel;
use crate::spinlock::{Spinlock, SpinlockToken};

pub fn sleep(_: Channel, lock: &Spinlock, tk: SpinlockToken) -> SpinlockToken {
    lock.release(tk);
    lock.acquire()
}

pub fn wakeup(_: Channel) {}
