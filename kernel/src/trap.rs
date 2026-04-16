use crate::arch::{Arch, CurrentArch, Intr};
use crate::drivers::{uart, virtio};
use crate::plic;
use crate::{memlayout, print};

core::arch::global_asm!(include_str!("asm/kernelvec.S"), kernel_trap = sym kernel_trap);

pub fn trapinithart() {
    CurrentArch::set_trap_vector(kernel_trap);
}

fn kernel_trap() {
    CurrentArch::kernel_trap();
}

// Check if it's an external interrupt or software interrupt, and handle it.
// Returns 2 if timer interrupt, 1 if other device, 0 if not recognized.
fn dev_intr() -> u8 {
    match CurrentArch::dev_intr() {
        Intr::Timer => 2,
        Intr::External => {
            let irq = plic::plic_claim();
            if irq == memlayout::UART0_IRQ {
                uart::uart_intr();
            } else if irq == memlayout::VIRTIO0_IRQ {
                virtio::virtio_disk_init();
            } else if irq != 0 {
                print::println!("Unexpected interrupt {irq}");
            }
            // The PLIC allows each device to raise at most one interrupt at a
            // time; tell the PLIC the device is now allowed to interrupt again.
            if irq != 0 {
                plic::plic_complete(irq);
            }
            1
        }
        Intr::Unrecognized => 0,
    }
}
