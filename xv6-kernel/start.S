    .section .text.entry
    .globl _start
_start:
    /* set up stack */
    la sp, __stack_top

    /* clear .bss */
    la t0, __bss_start
    la t1, __bss_end
1:
    bgeu t0, t1, 2f
    sd   zero, 0(t0)
    addi t0, t0, 8
    j    1b
2:
    /* jump to Rust */
    call kmain

3:  j 3b
