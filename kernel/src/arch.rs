mod riscv;

pub trait Arch {
    fn start(_: fn() -> !) -> !;
    fn cpuid() -> usize;
    fn interrupts_enabled() -> bool;
    fn enable_interrupts();
    fn disable_interrupts();
    fn set_trap_vector(_: fn());
    /// Halt the CPU until the next interrupt. If interrupts are disabled, this
    /// will halt indefinitely.
    fn halt() -> !;
    fn page_size() -> usize;
}

pub type CurrentArch = riscv::RiscVArch;

#[inline(always)]
pub fn ptr_address<T>(ptr: *const T) -> usize {
    ptr as usize
}

#[inline(always)]
pub fn pg_round_up(sz: usize) -> usize {
    (sz + CurrentArch::page_size() - 1) & !(CurrentArch::page_size() - 1)
}

#[inline(always)]
pub fn pg_round_down(sz: usize) -> usize {
    sz & !(CurrentArch::page_size() - 1)
}
