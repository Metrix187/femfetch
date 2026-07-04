/* femos i686 loads apps at a fixed physical window (paging off).
 * keep in sync with APP_LOAD_BASE in femos kernel/src/appload.c */
ENTRY(_start)

SECTIONS {
    . = 0x1800000;

    .text : { *(.text .text.*) }
    .rodata : { *(.rodata .rodata.*) }
    .data : { *(.data .data.*) }
    .bss : { *(COMMON) *(.bss .bss.*) }

    /DISCARD/ : {
        *(.eh_frame .eh_frame.*)
        *(.note .note.*)
        *(.comment)
    }
}
