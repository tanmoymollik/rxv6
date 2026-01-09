mod channel;
mod console;
mod kalloc;
mod printf;
mod proc;
mod spinlock;
mod uart;
mod vm;

use core::hint::spin_loop;
use core::sync::atomic::{AtomicBool, Ordering, fence};

static STARTED: AtomicBool = AtomicBool::new(false);

pub fn kmain() -> ! {
    if proc::cpuid() == 0 {
        console::consoleinit();
        printf::printfinit();
        crate::printf!("\n");
        crate::printf!("xv6 kernel is booting\n");
        crate::printf!("\n");
        kalloc::kinit(); // physical page allocator
        vm::kvminit(); // create kernel page table
        vm::kvminithart(); // turn on paging
        // procinit(); // process table
        // trapinit(); // trap vectors
        // trapinithart(); // install kernel trap vector
        // plicinit(); // set up interrupt controller
        // plicinithart(); // ask PLIC for device interrupts
        // binit(); // buffer cache
        // iinit(); // inode table
        // fileinit(); // file table
        // virtio_disk_init(); // emulated hard disk
        // userinit(); // first user process

        fence(Ordering::SeqCst);
        STARTED.store(true, Ordering::SeqCst);
    } else {
        while !STARTED.load(Ordering::SeqCst) {
            spin_loop();
        }
        fence(Ordering::SeqCst);
        //println!("hart {} starting\n", proc::cpuid());
    }
    loop {}
}
