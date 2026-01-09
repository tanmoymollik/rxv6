use paste::paste;

// generate assembly for reading csr.
// method is available as r_<csr_name>() -> usize
macro_rules! define_read_csr {
    ($name:ident) => {
        core::arch::global_asm!(concat!(
            ".section .text\n",
            // function global directive
            concat!(".global r_", stringify!($name), "\n"),
            // function name
            concat!("r_", stringify!($name), ":\n"),
            // read csr
            concat!("    csrr a0, ", stringify!($name), "\n"),
            // return
            "    ret\n"
        ));
        paste! {
        unsafe extern "C" {
            pub fn [<r_ $name>]() -> usize;
        }
        }
    };
}

// generate assembly for writing csr.
// method is available as w_<csr_name>(val: usize)
macro_rules! define_write_csr {
    ($name:ident) => {
        core::arch::global_asm!(concat!(
            ".section .text\n",
            // function global directive
            concat!(".global w_", stringify!($name), "\n"),
            // function name
            concat!("w_", stringify!($name), ":\n"),
            // new csr value in a0, write to csr
            concat!("    csrw ", stringify!($name), ", a0\n"),
            // return
            "    ret\n"
        ));
        paste! {
        unsafe extern "C" {
            pub fn [<w_ $name>](val: usize);
        }
        }
    };
}

// which hart (core) is this?
define_read_csr!(mhartid);

// machine Status Register, mstatus
pub const MSTATUS_MPP_MASK: usize = 3 << 11; // previous mode.
//pub const MSTATUS_MPP_M: usize = 3 << 11;
pub const MSTATUS_MPP_S: usize = 1 << 11;
//pub const MSTATUS_MPP_U: usize = 0 << 11;

define_read_csr!(mstatus);
define_write_csr!(mstatus);

// machine exception program counter, holds the instruction address to which a
// return from exception will go.
define_write_csr!(mepc);

// Supervisor Status Register, sstatus
pub const SSTATUS_SPP: usize = 1 << 8; // Previous mode, 1=Supervisor, 0=User
pub const SSTATUS_SPIE: usize = 1 << 5; // Supervisor Previous Interrupt Enable
pub const SSTATUS_UPIE: usize = 1 << 4; // User Previous Interrupt Enable
pub const SSTATUS_SIE: usize = 1 << 1; // Supervisor Interrupt Enable
pub const SSTATUS_UIE: usize = 1 << 0; // User Interrupt Enable

define_read_csr!(sstatus);
define_write_csr!(sstatus);

// use riscv's sv39 page table scheme.
const SATP_SV39: usize = 8 << 60;

#[inline(always)]
pub fn make_satp(pagetable: usize) -> usize {
    SATP_SV39 | (pagetable >> 12)
}

// supervisor address translation and protection;
// holds the address of the page table.
define_write_csr!(satp);

// Machine Exception Delegation
define_write_csr!(medeleg);

// Machine Interrupt Delegation
define_write_csr!(mideleg);

// Supervisor Interrupt Enable
pub const SIE_SEIE: usize = 1 << 9; // external
pub const SIE_STIE: usize = 1 << 5; // timer

define_read_csr!(sie);
define_write_csr!(sie);

// Physical Memory Protection
define_write_csr!(pmpcfg0);
define_write_csr!(pmpaddr0);

// enable device interrupts
#[inline(always)]
pub fn intr_on() {
    unsafe { w_sstatus(r_sstatus() | SSTATUS_SIE) }
}

// disable device interrupts
#[inline(always)]
pub fn intr_off() {
    unsafe { w_sstatus(r_sstatus() & !SSTATUS_SIE) }
}

// are device interrupts enabled?
#[inline(always)]
pub fn intr_get() -> bool {
    unsafe { (r_sstatus() & SSTATUS_SIE) != 0 }
}

// read and write tp, the thread pointer, which xv6 uses to hold
// this core's hartid (core number), the index into cpus[] in proc.rs.
#[inline(always)]
pub fn r_tp() -> usize {
    let tp: usize;
    unsafe {
        core::arch::asm!(
            "mv {}, tp",
            out(reg) tp)
    }
    tp
}

#[inline(always)]
pub fn w_tp(val: usize) {
    unsafe {
        core::arch::asm!(
            "mv tp, {}",
            in(reg) val
        );
    }
}

// Machine-mode Interrupt Enable
pub const MIE_STIE: usize = 1 << 5; // supervisor timer
define_read_csr!(mie);
define_write_csr!(mie);

// Machine Environment Configuration Register
define_read_csr!(menvcfg);
define_write_csr!(menvcfg);

// Machine-mode Counter-Enable
define_read_csr!(mcounteren);
define_write_csr!(mcounteren);

// Supervisor Timer Comparison Register
define_write_csr!(stimecmp);

// machine-mode cycle counter
define_read_csr!(time);

// Flush the TLB.
#[inline(always)]
pub fn sfence_vma() {
    // the zero, zero means flush all TLB entries.
    unsafe {
        core::arch::asm!("sfence.vma zero, zero");
    }
}

pub const PGSIZE: usize = 4096; // bytes per page
pub const PGSHIFT: usize = 12; // bits of offset within a page

#[inline(always)]
pub fn pgroundup(sz: usize) -> usize {
    (sz + PGSIZE - 1) & !(PGSIZE - 1)
}

#[inline(always)]
pub fn pgrounddown(sz: usize) -> usize {
    sz & !(PGSIZE - 1)
}

pub const PTE_V: usize = 1 << 0; // valid
pub const PTE_R: usize = 1 << 1;
pub const PTE_W: usize = 1 << 2;
pub const PTE_X: usize = 1 << 3;
pub const PTE_U: usize = 1 << 4; // user can access

// shift a physical address to the right place for a PTE.
#[inline(always)]
pub fn pa2pte(pa: usize) -> usize {
    (pa >> 12) << 10
}

#[inline(always)]
pub fn pte2pa(pa: usize) -> usize {
    (pa >> 10) << 12
}

#[inline(always)]
pub fn pte_flags(pte: usize) -> usize {
    pte & 0x3ff
}

// extract the three 9-bit page table indices from a virtual address.
pub const PXMASK: usize = 0x1FF; // 9 bits

#[inline(always)]
pub fn pxshift(level: usize) -> usize {
    PGSHIFT + 9 * level
}

#[inline(always)]
pub fn px(level: usize, va: usize) -> usize {
    (va >> pxshift(level)) & PXMASK
}

// one beyond the highest possible virtual address.
// MAXVA is actually one bit less than the max allowed by
// Sv39, to avoid having to sign-extend virtual addresses
// that have the high bit set.
pub const MAXVA: usize = 1 << (9 + 9 + 9 + 12 - 1);
