use crate::spinlock::Spinlock;
use kernelapi::fs::BSIZE;

pub struct Buf {
    // Has the data been read from disk?
    pub valid: bool,
    // Does disk "own" buf?
    pub disk: bool,
    dev: u32,
    pub blockno: u32,
    refcnt: u32,
    pub data: [u8; BSIZE],
    pub lock: Spinlock,
}

impl Buf {
    pub const fn new() -> Self {
        Buf {
            // Not used for now.
            valid: false,
            disk: false,
            // Not used for now.
            dev: 0,
            blockno: 0,
            refcnt: 0,
            data: [0; BSIZE],
            lock: Spinlock::new("buf_lock"),
        }
    }
}
