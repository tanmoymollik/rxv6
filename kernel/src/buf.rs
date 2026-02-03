use crate::spinlock::Spinlock;
use kernelapi::fs::BSIZE;

pub struct Buf {
    // Has the data been read from disk?
    valid: bool,
    // Does disk "own" buf?
    pub disk: bool,
    dev: u32,
    pub blockno: u32,
    refcnt: u32,
    pub data: [u8; BSIZE],
    lock: Spinlock,
}
