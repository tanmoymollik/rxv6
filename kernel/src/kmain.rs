use crate::arch::{Arch, CurrentArch};
use crate::kalloc;
use crate::plic;
use crate::print;
use crate::trap;

/// start::start() jumps here in supervisor mode on stack0 on all CPUs.
pub fn kmain() -> ! {
    if CurrentArch::cpuid() == 0 {
        crate::console::consoleinit();
        print::println!("\nrxv6 kernel booting\n");
        // Physical page allocator.
        kalloc::kinit();
        // Install kernel trap vector.
        trap::trapinithart();
        // Set up interrupt controller.
        plic::plicinit();
        // Ask PLIC for device interrupts.
        plic::plicinithart();
    } else {
        // Implement other CPU initialization here.
        CurrentArch::halt();
    }
    loop {}
}
