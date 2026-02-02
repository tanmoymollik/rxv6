use crate::arch::{Arch, CurrentArch};
use crate::param;

/// Per-CPU state.
#[derive(Copy, Clone)]
pub struct Cpu {
    /// Number of times disable interrupt has been called.
    pub noff: usize,
    /// Interrupt state before disable interrupt.
    pub intena: bool,
}

pub fn mycpu() -> &'static mut Cpu {
    unsafe { &mut CPUS[CurrentArch::cpuid()] }
}

// Can be indexed using CurrentArch::mycpu() to get current CPU state.
// Doesn't need a lock, as we have one instance per CPU.
static mut CPUS: [Cpu; param::NCPU] = [Cpu {
    noff: 0,
    intena: true,
}; param::NCPU];
