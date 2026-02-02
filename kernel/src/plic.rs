use core::mem;

use crate::arch::{Arch, CurrentArch};
use crate::memlayout;

pub fn plicinit() {
    write_reg(memlayout::plic_device_priority(memlayout::UART0_IRQ), 1);
    write_reg(memlayout::plic_device_priority(memlayout::VIRTIO0_IRQ), 1);
}

pub fn plicinithart() {
    let hart = CurrentArch::cpuid();

    // Set enable bits for this hart's S-mode for the uart and virtio disk.
    write_reg(
        memlayout::plic_senable(hart),
        (1 << memlayout::UART0_IRQ) | (1 << memlayout::VIRTIO0_IRQ),
    );
    // Set this hart's S-mode priority threshold to 0.
    write_reg(memlayout::plic_spriority(hart), 0);
}

// Ask the PLIC what interrupt we should serve.
pub fn plic_claim() -> u32 {
    let hart = CurrentArch::cpuid();
    let irq = read_reg(memlayout::plic_sclaim(hart));
    return irq;
}

// tell the PLIC we've served this IRQ.
pub fn plic_complete(irq: u32) {
    let hart = CurrentArch::cpuid();
    write_reg(memlayout::plic_sclaim(hart), irq);
}

fn write_reg(r: usize, val: u32) {
    unsafe { (r as *mut u32).write_volatile(val) };
}

fn read_reg(r: usize) -> u32 {
    unsafe { (r as *const u32).read_volatile() }
}
