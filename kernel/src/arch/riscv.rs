use core::arch::asm;

pub struct RiscVArch;

impl super::Arch for RiscVArch {
    fn start(main: fn() -> !) -> ! {
        // Set Machine Exception Program Counter to main(), for mret.
        w_mepc(main as usize);

        // Disable paging for now.
        w_satp(0);

        // Delegate all interrupts and exceptions to supervisor mode.
        w_medeleg(0xffff);
        w_mideleg(0xffff);

        // Enable supervisor mode timer interrupts.
        // This must be enabled in both mie and sie.
        w_mie(r_mie() | MIE_STIE);
        // Enable external and timer interrupt.
        w_sie(r_sie() | SIE_SEIE | SIE_STIE);
        // Enable the sstc extension (required to use stimecmp).
        w_menvcfg(r_menvcfg() | MENVCFG_STCE);
        // Allow supervisor to access time, stimecmp register.
        w_mcounteren(r_mcounteren() | MCOUNTEREN_TM);
        // Ask for the very first timer interrupt.
        w_stimecmp(r_time() + 1000000);

        // Configure Physical Memory Protection to give supervisor mode
        // access to all of physical memory.
        w_pmpaddr0(0x3fffffffffffff);
        w_pmpcfg0(0xf);

        // Keep each CPU's hartid in its tp register, for cpuid().
        w_tp(r_mhartid());

        let mut x = r_mstatus();
        // Clear Machine Previous Privilege mode.
        x &= !MSTATUS_MPP_MASK;
        // Set Machine Previous Privilege mode to Supervisor, for mret.
        x |= MSTATUS_MPP_S;
        w_mstatus(x);
        unsafe {
            // Switch to supervisor mode and jump to main().
            asm!("mret", options(noreturn));
        }
    }

    #[inline(always)]
    fn cpuid() -> usize {
        r_tp()
    }

    #[inline(always)]
    fn interrupts_enabled() -> bool {
        (r_sstatus() & SSTATUS_SIE) != 0
    }

    #[inline(always)]
    fn enable_interrupts() {
        w_sstatus(r_sstatus() | SSTATUS_SIE);
    }

    #[inline(always)]
    fn disable_interrupts() {
        w_sstatus(r_sstatus() & !SSTATUS_SIE);
    }

    #[inline(always)]
    fn set_trap_vector(trapvec: fn()) {
        w_stvec(trapvec as usize);
    }

    #[inline(always)]
    fn halt() -> ! {
        loop {
            // On RISC-V, 'wfi' (Wait For Interrupt) saves power while looping.
            unsafe { core::arch::asm!("wfi") };
        }
    }

    #[inline(always)]
    fn page_size() -> usize {
        4096
    }
}

// Generate assembly for reading a csr.
// Method is available as r_<csr_name>() -> usize
macro_rules! define_read_csr {
    ($csr_name: ident) => {
        paste::paste! {
            #[inline(always)]
            fn [<r_ $csr_name>]() -> usize {
                let ret;
                unsafe { asm!(concat!("csrr {}, ", stringify!($csr_name)), out(reg) ret); }
                ret
            }
        }
    };
}

// Generate assembly for writing to a csr.
// Method is available as w_<csr_name>(val: usize)
macro_rules! define_write_csr {
    ($csr_name: ident) => {
        paste::paste! {
            #[inline(always)]
            fn [<w_ $csr_name>](val: usize) {
                unsafe { asm!(concat!("csrw ", stringify!($csr_name), ", {}"), in(reg) val); }
            }
        }
    };
}

// Machine hart id, mhartid.
define_read_csr!(mhartid);

// Machine Status register, mstatus.
define_read_csr!(mstatus);
define_write_csr!(mstatus);
// Machine Previous Privilege mode mask.
const MSTATUS_MPP_MASK: usize = 3 << 11;
// Machine Previous Privilege mode supervisor.
const MSTATUS_MPP_S: usize = 1 << 11;

// Machine Exception Program Counter, mepc.
// Holds the instruction address to which a return from exception will go.
define_write_csr!(mepc);

// Machine Exception Delegation, medeleg.
define_write_csr!(medeleg);

// Machine Interrupt Delegation, mideleg.
define_write_csr!(mideleg);

// Machine Interrupt Enable, mie.
define_read_csr!(mie);
define_write_csr!(mie);
// Machine enable supervisor timer interrupt.
const MIE_STIE: usize = 1 << 5;

// Machine Environment Configuration Register, menvcfg.
define_read_csr!(menvcfg);
define_write_csr!(menvcfg);
// Machine enable stimecmp.
const MENVCFG_STCE: usize = 1 << 63;

// Machine Counter Enable for supervisor mode, mcounteren.
define_read_csr!(mcounteren);
define_write_csr!(mcounteren);
// Machine enable time for supervisor mode.
const MCOUNTEREN_TM: usize = 2;

// Machine cycle counter, time.
define_read_csr!(time);

// Supervisor Status register, sstatus.
define_read_csr!(sstatus);
define_write_csr!(sstatus);
// Supervisor interrupt enable.
const SSTATUS_SIE: usize = 1 << 1;

// Supervisor Interrupt Enable, sie.
define_read_csr!(sie);
define_write_csr!(sie);
// Enable supervisor external interrupt.
pub const SIE_SEIE: usize = 1 << 9;
// Enable supervisor timer interrupt.
pub const SIE_STIE: usize = 1 << 5;

// Supervisor Address Translation and Protection, satp.
// Holds the address of the page table.
define_write_csr!(satp);

// Supervisor Timer Comparison, stimecmp.
define_write_csr!(stimecmp);

// Supervisor Trap Vector, stvec.
define_write_csr!(stvec);

// Physical Memory Protection csrs.
define_write_csr!(pmpaddr0);
define_write_csr!(pmpcfg0);

// Read and Write tp, the thread pointer, which rxv6 uses to hold
// this core's hartid (core number).
#[inline(always)]
fn r_tp() -> usize {
    let tp: usize;
    unsafe {
        asm!(
            "mv {}, tp",
            out(reg) tp)
    }
    tp
}

#[inline(always)]
fn w_tp(val: usize) {
    unsafe {
        asm!(
            "mv tp, {}",
            in(reg) val
        );
    }
}
