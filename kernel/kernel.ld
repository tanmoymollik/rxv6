OUTPUT_ARCH( "riscv" )
ENTRY( _entry )

SECTIONS
{
    /*
     * Ensure that src/asm/entry.S:_entry is at 0x80000000,
     * where qemu's -kernel jumps.
     */
    . = 0x80000000;

    .text : {
        /* Ensure the entry point is at the very beginning. */
        KEEP(*(.text.entry))
        *(.text .text.*)
        . = ALIGN(0x1000);
        PROVIDE(etext = .);
    }

    .rodata : {
        . = ALIGN(16);
        /* Do not need to distinguish this from .rodata. */
        *(.srodata .srodata.*)
        . = ALIGN(16);
        *(.rodata .rodata.*)
    }

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
    
    /*
     * Used by src/asm/entry.S to set up stack before entering the
     * scheduling phase.
     * NCPU must be defined as an environment variable.
     */
    .stack0 (NOLOAD) : {
        . = ALIGN(0x1000);
        . += NCPU * 0x1000;
        PROVIDE(stack0 = .);
    }

    PROVIDE(end = .);
}