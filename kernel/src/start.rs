#![no_std]
#![no_main]
// Enable modern macro syntax.
#![feature(decl_macro)]

mod arch;
mod buf;
mod channel;
mod console;
mod cpu;
mod drivers;
mod elf;
mod kalloc;
mod kmain;
mod kstate;
mod kutils;
mod memlayout;
mod param;
mod plic;
mod print;
mod proc;
mod spinlock;
mod trap;

core::arch::global_asm!(include_str!("asm/entry.S"), start = sym start);

// asm/entry.S:_entry jumps here in machine mode on stack0.
fn start() -> ! {
    use arch::Arch;
    arch::CurrentArch::start(kmain::kmain)
}
