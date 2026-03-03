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
    __text_start = .;
    KEEP(*(.text.entry))
    *(.text .text.*)
    __text_end = .;
  }

  .rodata : ALIGN(4K) {
    __rodata_start = .;
    *(.rodata .rodata.*)
  }

  .data : ALIGN(4K) {
    __data_start = .;
    *(.data .data.*)
  }

  .bss : ALIGN(4K) {
    __bss_start = .;
    *(.bss .bss.*)
    *(COMMON)
    __bss_end = .;
  }

  . = ALIGN(4K);
  __available_memory_top = .;
  . = . + 0x40000; /* 64 * 4K to fit 64 pages */ 
  __available_memory_bottom = .;

  . = ALIGN(16);
  __kernel_stack_bottom = .;
  . = . + 0x1000;      /* reserve 4K */
  __kernel_stack_top = .;

  . = ALIGN(16);
  __stack_bottom = .;
  . = . + 0x1000;      /* reserve 4K */
  __stack_top = .;

  . = ALIGN(4K);
  __memory_start = .;
}

