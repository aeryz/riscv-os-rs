# An experimental RISC-V OS

This OS is mostly for learning purposes. It started as a challenge but one of my primary goals is to also turn this into a series of videos and blog posts.

I initially wanted to re-implement the xv6 kernel in Rust but I didn't want to just copy code and instead actually try to live through the pain of building everything from scratch.

## Running

Right now, I only have the docs for nix users:

### Enter the devshell
Use `direnv`:
```
direnv allow
```

Or just:
```
nix develop
```

### Run the OS
```
qemu-system-riscv64 \
  -nographic \
  -machine virt \
  -bios target/riscv64gc-unknown-none-elf/release/kernel
```

## Non-exhaustive list of the roadmap

TBD

## Resources

1. Huge shoutout to the [OSTEP book](https://pages.cs.wisc.edu/~remzi/OSTEP/) that let me grasp most of the OS concepts. It is a very easy to read book so I strongly recommend it. (don't forget to support the author if you can)
2. The blog posts of [Uros Popovic](https://popovicu.com/posts/bare-metal-programming-risc-v/) made it easier to bootstrap the project by explaining the qemu RISC-V internals.
3. The official [RISC-V specification](https://docs.riscv.org/reference/isa/) is very helpful to have the full layout of the registers and basically anything related to the hardware.
4. Very comprehensive [RISC-V course](https://www.youtube.com/watch?v=VEQL5bJeWB0&list=PLbtzT1TYeoMiKup6aoQc3V_d7OvOKc3P5&index=1) by Harry H. Porter (what a cool name). Reading the specification is not easy and can feel a bit dry. This helps you grasp the RISC-V assembly, trap handling, CSR's, etc. It's basically RISC-V spec but for humans.
5. I use ChatGPT only for asking questions about the risc-v spec when I'm stuck. LLMs are great tools for fetching you a specific information out of huge documents. But note that, it certainly won't help to let the AI code for you in this case. The learning comes from suffering.
6. [xv6-kernel documentation by MIT](https://pdos.csail.mit.edu/6.828/2020/xv6/book-riscv-rev1.pdf) I skim through the documentation to see their choice of algorithms. Would be a great source if you prefer to follow this course with it's source code entirely.

## Contribution
I'm not an expert at all. I'm just learning things by doing it. So, feel free to drop an issue if you:
- spot an error,
- think that there is a better way of doing things,
- have any questions. (issue labeled as "question")

