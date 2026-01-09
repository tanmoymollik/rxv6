// don't link the Rust standard library
#![no_std]
// disable all Rust-level entry points
#![no_main]

mod kmain;
mod memlayout;
mod param;
mod riscv;

// Define the stack size (4KB * num_cores)
const STACK_SIZE: usize = 4096 * param::NCPU;

// Create a wrapper struct to enforce alignment
#[repr(C, align(16))]
struct Stack([u8; STACK_SIZE]);

// "static mut" places it in the .bss section (because it is initialized to 0).
#[unsafe(no_mangle)]
static mut STACK0: Stack = Stack([0; STACK_SIZE]);

core::arch::global_asm!(include_str!("asm/entry.S"), stack0 = sym STACK0, start = sym start);
core::arch::global_asm!(include_str!("asm/trampoline.S"), trapframe = const memlayout::TRAPFRAME);

fn start() {
    unsafe {
        // set M Previous Privilege mode to Supervisor, for mret.
        let mut x = riscv::r_mstatus();
        x &= !riscv::MSTATUS_MPP_MASK;
        x |= riscv::MSTATUS_MPP_S;
        riscv::w_mstatus(x);

        // set M Exception Program Counter to kmain, for mret.
        riscv::w_mepc(kmain::kmain as *const () as usize);

        // disable paging for now.
        riscv::w_satp(0);

        // delegate all interrupts and exceptions to supervisor mode.
        riscv::w_medeleg(0xffff);
        riscv::w_mideleg(0xffff);
        riscv::w_sie(riscv::r_sie() | riscv::SIE_SEIE | riscv::SIE_STIE);

        // configure Physical Memory Protection to give supervisor mode
        // access to all of physical memory.
        riscv::w_pmpaddr0(0x3fffffffffffff);
        riscv::w_pmpcfg0(0xf);

        // ask for clock interrupts.
        timerinit();

        // keep each CPU's hartid in its tp register, for cpuid().
        let id = riscv::r_mhartid();
        riscv::w_tp(id);

        // switch to supervisor mode and jump to kmain().
        core::arch::asm!("mret", options(nomem, nostack));
    }
}

unsafe fn timerinit() {
    unsafe {
        // enable supervisor-mode timer interrupts.
        riscv::w_mie(riscv::r_mie() | riscv::MIE_STIE);

        // enable the sstc extension (i.e. stimecmp).
        riscv::w_menvcfg(riscv::r_menvcfg() | (1 << 63));

        // allow supervisor to use stimecmp and time.
        riscv::w_mcounteren(riscv::r_mcounteren() | 2);

        // ask for the very first timer interrupt.
        riscv::w_stimecmp(riscv::r_time() + 1000000);
    }
}
