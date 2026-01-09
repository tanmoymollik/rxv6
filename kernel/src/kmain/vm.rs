use super::kalloc;
use super::proc;
use crate::memlayout::*;
use crate::riscv::*;

enum Sbrk {
    Eager,
    Lazy,
}
// #include "param.h"
// #include "types.h"
// #include "memlayout.h"
// #include "elf.h"
// #include "riscv.h"
// #include "defs.h"
// #include "spinlock.h"
// #include "proc.h"
// #include "fs.h"

unsafe extern "C" {
    // kernel.ld sets this to end of kernel code.
    static etext: [u8; 0];
    // trampoline.S
    static trampoline: [u8; 0];
}

pub type PageTableEntry = usize;
pub type PageTable = [PageTableEntry; 512]; // 512 PTEs

/*
 * the kernel's page table.
 */
static mut KERNEL_PAGETABLE: usize = 0;

// Make a direct-map page table for the kernel.
fn kvmmake() -> *mut PageTable {
    let kpgtbl = kalloc::kalloc() as *mut PageTable;
    unsafe {
        core::ptr::write_bytes(kpgtbl as *mut u8, 0, PGSIZE);
    }
    let kpgtbl = unsafe { &mut (*kpgtbl) };

    // uart registers
    kvmmap(kpgtbl, UART0, UART0, PGSIZE, PTE_R | PTE_W);

    // virtio mmio disk interface
    kvmmap(kpgtbl, VIRTIO0, VIRTIO0, PGSIZE, PTE_R | PTE_W);

    // PLIC
    kvmmap(kpgtbl, PLIC, PLIC, 0x4000000, PTE_R | PTE_W);

    let etext_addr = unsafe { etext.as_ptr() as usize };
    // map kernel text executable and read-only.
    kvmmap(
        kpgtbl,
        KERNBASE,
        KERNBASE,
        etext_addr - KERNBASE,
        PTE_R | PTE_X,
    );

    // map kernel data and the physical RAM we'll make use of.
    kvmmap(
        kpgtbl,
        etext_addr,
        etext_addr,
        PHYSTOP - etext_addr,
        PTE_R | PTE_W,
    );

    // map the trampoline for trap entry/exit to
    // the highest virtual address in the kernel.
    let trampoline_addr = unsafe { trampoline.as_ptr() as usize };
    kvmmap(kpgtbl, TRAMPOLINE, trampoline_addr, PGSIZE, PTE_R | PTE_X);

    // allocate and map a kernel stack for each process.
    proc::proc_mapstacks(kpgtbl);

    return kpgtbl;
}

// add a mapping to the kernel page table.
// only used when booting.
// does not flush TLB or enable paging.
fn kvmmap(kpgtbl: &mut PageTable, va: usize, pa: usize, sz: usize, perm: usize) {
    if !mappages(kpgtbl, va, sz, pa, perm) {
        panic!("kvmmap");
    }
}

// Initialize the kernel_pagetable, shared by all CPUs.
pub fn kvminit() {
    unsafe {
        KERNEL_PAGETABLE = kvmmake() as usize;
    }
}

// Switch the current CPU's h/w page table register to
// the kernel's page table, and enable paging.
pub fn kvminithart() {
    // wait for any previous writes to the page table memory to finish.
    sfence_vma();

    unsafe {
        w_satp(make_satp(KERNEL_PAGETABLE));
    }

    // flush stale entries from the TLB.
    sfence_vma();
}

// Return the address of the PTE in page table pagetable
// that corresponds to virtual address va.  If alloc = true,
// create any required page-table pages.
//
// The risc-v Sv39 scheme has three levels of page-table
// pages. A page-table page contains 512 64-bit PTEs.
// A 64-bit virtual address is split into five fields:
//   39..63 -- must be zero.
//   30..38 -- 9 bits of level-2 index.
//   21..29 -- 9 bits of level-1 index.
//   12..20 -- 9 bits of level-0 index.
//    0..11 -- 12 bits of byte offset within the page.
fn walk(pagetable: &mut PageTable, va: usize, alloc: bool) -> Option<*mut PageTableEntry> {
    if va >= MAXVA {
        panic!("walk");
    }

    let mut pagetable_ptr = pagetable as *mut PageTable;
    for level in (1..=2).rev() {
        unsafe {
            let pte = &mut (*pagetable_ptr)[px(level, va)] as *mut PageTableEntry;
            if *pte & PTE_V > 0 {
                // valid entry
                pagetable_ptr = pte2pa(*pte) as *mut PageTable;
            } else {
                if !alloc {
                    return None;
                }
                pagetable_ptr = kalloc::kalloc() as *mut PageTable;
                core::ptr::write_bytes(pagetable_ptr as *mut u8, 0, PGSIZE);
                *pte = pa2pte(pagetable_ptr as usize) | PTE_V;
            }
        }
    }
    return unsafe { Some(&mut (*pagetable_ptr)[px(0, va)] as *mut PageTableEntry) };
}

// Create PTEs for virtual addresses starting at va that refer to
// physical addresses starting at pa.
// va and size MUST be page-aligned.
// Returns true on success, false if walk() couldn't
// allocate a needed page-table page.
fn mappages(pagetable: &mut PageTable, va: usize, size: usize, mut pa: usize, perm: usize) -> bool {
    if (va % PGSIZE) != 0 {
        panic!("mappages: va not aligned");
    }

    if (size % PGSIZE) != 0 {
        panic!("mappages: size not aligned");
    }

    if size == 0 {
        panic!("mappages: size");
    }

    for _a in (0..size).step_by(PGSIZE) {
        let a = va + _a;
        match walk(pagetable, a, true) {
            Some(pte) => unsafe {
                if *pte & PTE_V > 0 {
                    panic!("mappages: remap");
                }
                *pte = pa2pte(pa) | perm | PTE_V;
                pa += PGSIZE;
            },
            None => return false,
        }
    }

    return true;
}
