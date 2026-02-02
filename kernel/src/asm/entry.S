# qemu -kernel loads the kernel at 0x80000000 and causes
# each hart (i.e. CPU) to jump there. kernel.ld causes the
# following code to be placed at 0x80000000.
.section .text.entry
.extern stack0
.global _entry
_entry:
    # set up a stack for rust.
    # stack0 is declared in kernel.ld, with a 4096-byte
    # stack per CPU.
    # sp = stack0 + ((hartid + 1) * 4096)
    la sp, stack0
    csrr a0, mhartid
    addi a0, a0, 1
    slli a0, a0, 12        # a0 = a0 * 4096
    add sp, sp, a0
    call {start}           # jump to start() in start.rs
.Lspin:
    j .Lspin
