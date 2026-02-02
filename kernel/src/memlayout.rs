/// Physical Memory Layout
///
/// Qemu -machine virt is set up like this, based on qemu's hw/riscv/virt.c:
///
/// 0x00001000 -- boot ROM, provided by qemu
/// 0x02000000 -- CLINT
/// 0x0C000000 -- PLIC
/// 0x10000000 -- uart0
/// 0x10001000 -- virtio disk
/// 0x80000000 -- qemu's boot ROM loads the kernel here,
///             then jumps here.
/// Unused RAM after 0x80000000.
///
/// The kernel uses physical memory thus:
/// 0x80000000 -- asm/entry.S
/// Kernel text and data
/// end -- start of kernel page allocation area
/// PHYSTOP -- end RAM used by the kernel

/// Qemu puts UART registers here in physical memory.
pub const UART0: usize = 0x10000000;
pub const UART0_IRQ: u32 = 10;

/// Virtio mmio interface.
pub const VIRTIO0: usize = 0x10001000;
pub const VIRTIO0_IRQ: u32 = 1;

/// Qemu puts platform-level interrupt controller (PLIC) here.
pub const PLIC: usize = 0x0c000000;
pub const PLIC_PRIORITY: usize = PLIC + 0x0;
pub const PLIC_PENDING: usize = PLIC + 0x1000;

/// Returns address of the plic priority register for the device irq
/// (e.g. `UART0_IRQ`).
pub fn plic_device_priority(irq: u32) -> usize {
    PLIC + (irq as usize) * 4
}

/// Returns address of the plic enable register for supervisor mode for
/// `hart` (odd contexts).
#[inline(always)]
pub fn plic_senable(hart: usize) -> usize {
    PLIC + 0x2080 + (hart) * 0x100
}

/// Returns address of the plic priority register for supervisor mode for
/// `hart` (odd contexts).
#[inline(always)]
pub fn plic_spriority(hart: usize) -> usize {
    PLIC + 0x201000 + (hart) * 0x2000
}

/// Returns address of the plic claim register for supervisor mode for
/// `hart` (odd contexts).
#[inline(always)]
pub fn plic_sclaim(hart: usize) -> usize {
    PLIC + 0x201004 + (hart) * 0x2000
}

// The kernel expects there to be RAM for use by the kernel and user pages from
// physical address 0x80000000 to `PHYSTOP`.
pub const KERNBASE: usize = 0x80000000;
pub const PHYSTOP: usize = KERNBASE + 128 * 1024 * 1024; // 128 MB
