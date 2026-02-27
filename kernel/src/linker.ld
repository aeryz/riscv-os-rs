OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS
{
  /* fw_dynamic is loaded at 0x80000000
     TODO: check why we load at 0x80200000
     and not some other address
  */
  . = 0x80200000;
  

  .text : ALIGN(4K) {
    KEEP(*(.text.entry))
    *(.text .text.*)
  }

  .rodata : ALIGN(4K) {
    *(.rodata .rodata.*)
  }

  .data : ALIGN(4K) {
    *(.data .data.*)
  }

  . = ALIGN(16);
  __stack_bottom = .;
  . = . + 0x4000;      /* reserve 16 KiB */
  __stack_top = .;
}

