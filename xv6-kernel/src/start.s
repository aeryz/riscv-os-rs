    .section .text.entry
    .globl _start
_start:
    /* set up stack */
    la sp, __stack_top
1:
    bgeu t0, t1, 2f
    sd   zero, 0(t0)
    addi t0, t0, 8
    j    1b
2:
    call kmain

3:  j 3b
