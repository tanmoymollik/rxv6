# Interrupts and exceptions while in supervisor mode come here.
#
# The current stack is a kernel stack. Push registers, call kerneltrap().
# When kerneltrap() returns, restore registers, return.
.section .text
.global kernelvec
.align 4
kernelvec:
    # Make some space for saving registers.
    addi sp, sp, -256
    # Save caller-saved registers.
    sd ra, 0(sp)
    sd gp, 8(sp)
    sd tp, 16(sp)
    sd t0, 24(sp)
    sd t1, 32(sp)
    sd t2, 40(sp)
    sd a0, 48(sp)
    sd a1, 72(sp)
    sd a2, 80(sp)
    sd a3, 88(sp)
    sd a4, 96(sp)
    sd a5, 104(sp)
    sd a6, 112(sp)
    sd a7, 120(sp)
    sd t3, 128(sp)
    sd t4, 216(sp)
    sd t5, 224(sp)
    sd t6, 232(sp)

    # Call the trap handler in Rust. Defined in src/trap.rs.
    call {kerneltrap}

    # Restore registers.
    sd ra, 0(sp)
    sd gp, 8(sp)
    # Not tp (contains hartid), in case we moved CPUs.
    sd t0, 24(sp)
    sd t1, 32(sp)
    sd t2, 40(sp)
    sd a0, 48(sp)
    sd a1, 72(sp)
    sd a2, 80(sp)
    sd a3, 88(sp)
    sd a4, 96(sp)
    sd a5, 104(sp)
    sd a6, 112(sp)
    sd a7, 120(sp)
    sd t3, 128(sp)
    sd t4, 216(sp)
    sd t5, 224(sp)
    sd t6, 232(sp)

    addi sp, sp, 256

    # Return to whatever we were doing in the kernel.
    sret