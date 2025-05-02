SECTIONS
{
    RAM_BASE = 0x80000000;
    ENTRY(_reset_vector)
    ASSERT(_reset_vector == RAM_BASE, "Error: reset vector should be at RAM_BASE.")

    . = RAM_BASE;
    .text :
    {
        *(.text.reset_vector)
        *(.text*)
    }
    . = 0x80006000;
    .rodata :
    {
        *(.addresses.mac)
        *(.addresses.linklocal)
        *(.rodata*)
        *(.srodata*)
    }
    . = ALIGN(0x10);
    _data_begin = .;
    .data :
    {
        *(.data*)
        *(.sdata*)
    }
    _bss_begin = .;
    ASSERT(_bss_begin == _data_begin, "Error: .data is prohibited. Initial values of variables do not restore on reset.")
    .bss :
    {
        *(.bss*)
        *(.sbss*)
    }
    . = ALIGN(0x10);
    _bss_end = .;

    . = 0x80600000;
    .dma.router2cpu : {}

    /DISCARD/ :
    {
        *(.note.gnu.build-id)
    }

    . = 0x80800000;
    .entryPool : {}

    . = 0x80900000;
    .nextHopList : {}

    . = 0x80a00000;
    .dma.cpu2router : {}
    . = 0x80a10000;
    .dma.status : {}

    . = 0x80b00000;
    .dpy : {}

    . = 0x81000000;
    .nodePool : {}

    . = 0x82000000;
    .fib : {}
}
