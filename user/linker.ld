OUTPUT_ARCH( "riscv" )
ENTRY( main )

SECTIONS
{
    /*
     * Ensure that src/asm/entry.S:_entry is at 0x80000000,
     * where qemu's -kernel jumps.
     */
    . = 0x80000000;

    .text : {
        *(.text .text.*)
    }

    .rodata : {
        . = ALIGN(16);
        /* Do not need to distinguish this from .rodata. */
        *(.srodata .srodata.*)
        . = ALIGN(16);
        *(.rodata .rodata.*)
    }
    
    .eh_frame : {
        *(.eh_frame .eh_frame.*)
    }

    . = ALIGN(0x1000);
    .data : {
        . = ALIGN(16);
        /* Do not need to distinguish this from .data. */
        *(.sdata .sdata.*)
        . = ALIGN(16);
        *(.data .data.*)
    }

    .bss : {
        . = ALIGN(16);
        /* Do not need to distinguish this from .bss. */
        *(.sbss .sbss.*)
        . = ALIGN(16);
        *(.bss .bss.*)
    }
    
    PROVIDE(end = .);
}