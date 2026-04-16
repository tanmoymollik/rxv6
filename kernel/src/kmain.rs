use crate::arch::{self, Arch, CurrentArch};
use crate::drivers::virtio;
use crate::kalloc;
use crate::plic;
use crate::print::{self, println};
use crate::trap;

/// start::start() jumps here in supervisor mode on stack0 on all CPUs.
pub fn kmain() -> ! {
    if CurrentArch::cpuid() == 0 {
        crate::console::consoleinit();
        print::println!("\nrxv6 kernel booting\n");
        println!("end address: 0x{:x}", kalloc::end_addr());
        println!(
            "end address round up: 0x{:x}",
            arch::pg_round_up(kalloc::end_addr())
        );
        // Physical page allocator.
        kalloc::kinit();
        println!("kinit");
        // Install kernel trap vector.
        trap::trapinithart();
        println!("trapinithart");
        // Set up interrupt controller.
        plic::plicinit();
        println!("plicinit");
        // Ask PLIC for device interrupts.
        plic::plicinithart();
        println!("plicinithart");
        // TODO(): Move this to driver::virtio_disk_init.
        // Emulated hard disk.
        virtio::virtio_disk_init();
        println!("virtio");
    } else {
        // Implement other CPU initialization here.
        CurrentArch::halt();
    }
    loop {}
}
