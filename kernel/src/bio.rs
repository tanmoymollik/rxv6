use crate::buf::Buf;
use crate::drivers::virtio;
use crate::spinlock::SpinlockToken;

static mut BUF: Buf = Buf::new();

fn bget(_ /* dev= */: u32, blockno: u32) -> (*mut Buf, SpinlockToken) {
    unsafe {
        let buf = &raw mut BUF;
        let tk = (*buf).lock.acquire();
        (*buf).valid = false;
        (*buf).blockno = blockno;
        (buf, tk)
    }
}

// Returns a locked buf with the contents of the indicated block.
pub fn bread(dev: u32, blockno: u32) -> (&'static Buf, SpinlockToken) {
    let (buf, tk) = bget(dev, blockno);
    unsafe {
        if !(*buf).valid {
            virtio::disk_rw(buf as *mut Buf, false);
            (*buf).valid = true;
        }
        (&*buf, tk)
    }
}
