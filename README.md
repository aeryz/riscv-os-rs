# An experimental RISC-V OS

This OS is mostly for learning purposes. It started as a challenge but one of my primary goals is to also create turn this into a series of videos and blog posts.

I initially wanted to re-implement the xv6 kernel in Rust but I didn't want to just copy code and instead actually try to live through the pain of building everything from scratch.

## Non-exhaustive list of the roadmap

### Phase 1 (boot):
- [ ] Boot into the kernel in M-mode.
- [ ] Print to console using UART.
- [ ] Do the necessary adjustments to `mstatus`, setup the `pmp` and switch to S-mode.
- [ ] Setup a stub trap handler and switch to U-mode. Make sure `ecall` works.

### Phase 2 (prepare for userspace programs):
- [ ] Have a meaningful `ecall` similar to a `write` syscall in Linux, where the U-mode can print to console.
- [ ] Setup a basic page table. No allocator, just try to use page table'd accesses in the U-mode.
- [ ] Arrange the project and isolate the inline assembly into a reusable library.
- [ ] Make the trap handler properly handle the kernel/userspace traps and go back to the userspace code properly.
- [ ] Have a page allocator (haven't decided on the algorithm right now)

### Phase 3 (handle userspace programs):
- [ ] Create one userspace process.
- [ ] Add timer interrupt and experiment with yielding the execution to the kernel.
- [ ] Work with multiple processes with a basic round robin scheduler. (context switch).

### Phase 4 (filesystem):
TBD

### Phase 5 (more functionality from the POSIX):
TBD
