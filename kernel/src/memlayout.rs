// Physical memory layout

// qemu -machine virt is set up like this,
// based on qemu's hw/riscv/virt.c:
//
// 00001000 -- boot ROM, provided by qemu
// 02000000 -- CLINT
// 0C000000 -- PLIC
// 10000000 -- uart0
// 10001000 -- virtio disk
// 80000000 -- qemu's boot ROM loads the kernel here,
//             then jumps here.
// unused RAM after 80000000.

// the kernel uses physical memory thus:
// 80000000 -- asm/entry.S, then kernel text and data
// end -- start of kernel page allocation area
// PHYSTOP -- end RAM used by the kernel

use crate::riscv;

// qemu puts UART registers here in physical memory.
pub const UART0: usize = 0x10000000;
pub const UART0_IRQ: usize = 10;

// virtio mmio interface
pub const VIRTIO0: usize = 0x10001000;
pub const VIRTIO0_IRQ: usize = 1;

// qemu puts platform-level interrupt controller (PLIC) here.
pub const PLIC: usize = 0x0c000000;
pub const PLIC_PRIORITY: usize = PLIC + 0x0;
pub const PLIC_PENDING: usize = PLIC + 0x1000;

#[inline(always)]
pub fn plic_senable(hart: usize) -> usize {
    PLIC + 0x2080 + hart * 0x100
}

#[inline(always)]
pub fn plic_spriority(hart: usize) -> usize {
    PLIC + 0x201000 + hart * 0x2000
}

#[inline(always)]
pub fn plic_sclaim(hart: usize) -> usize {
    PLIC + 0x201004 + (hart) * 0x2000
}

// the kernel expects there to be RAM
// for use by the kernel and user pages
// from physical address 0x80000000 to PHYSTOP.
pub const KERNBASE: usize = 0x80000000;
pub const PHYSTOP: usize = KERNBASE + 128 * 1024 * 1024;

// map the trampoline page to the highest address,
// in both user and kernel space.
// Used in assembly.
pub const TRAMPOLINE: usize = riscv::MAXVA - riscv::PGSIZE;

// map kernel stacks beneath the trampoline,
// each surrounded by invalid guard pages.
// #define KSTACK(p) (TRAMPOLINE - ((p)+1)* 2*PGSIZE)

// User memory layout.
// Address zero first:
//   text
//   original data and bss
//   fixed-size stack
//   expandable heap
//   ...
//   TRAPFRAME (p->trapframe, used by the trampoline)
//   TRAMPOLINE (the same page as in the kernel)
// Used in assembly.
pub const TRAPFRAME: usize = TRAMPOLINE - riscv::PGSIZE;
