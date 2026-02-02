**[Work in progress]**

Currently working on finishing up the virtio device driver so that I can load my
first user program from a file system created by mkfs!
The file system is still at very minial state just to test virtio and elf
loading.

rxv6 is a re-implementation of the [xv6](https://github.com/mit-pdos/xv6-riscv.git)
educational operation system from MIT in rust. This implementation targets the
riscv architecture for development.

This is an educational project for having a hands on experience on two things
that I have been learning for some time now - rust and operating system concepts.

I am trying to keep the original code structure and comments as much as possible
so that it can be used as a 1:1 replacement for the xv6 operating system written
in a more memory safe language.

## Dependencies ##
- cargo-make
- riscv64gc toolchain
- qemu

## Changes ##
- file names are same but put in different folders
- main renamed to kmain because cargo can get confused with the name
- types.h removed. rust types used instead.
- Some c function-like defines like PGROUNDUP converted to rust macro.
- use core::ptr::write_bytes() instead of defining memset.
- panic method in printf converted to panic! macro in rust.
- Makefile converted to justfile.

## Behavior ##
- When one cpu panics the whole kernel is thought to be panicking. (Can another
cpu do recovery?)