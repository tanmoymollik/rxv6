use super::channel::Channel;
use super::spinlock::SpinLock;
use super::vm;
use crate::param;
use crate::riscv;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

// Per-CPU state.
pub struct Cpu {
    // The process running on this cpu, or null.
    proc: UnsafeCell<*mut Proc>,
    // swtch() here to enter scheduler().
    context: Context,
    // Depth of push_off() nesting.
    noff: UnsafeCell<usize>,
    // Were interrupts enabled before push_off()?
    intena: UnsafeCell<bool>,
}

impl Cpu {
    #[inline(always)]
    pub fn noff(&self) -> usize {
        unsafe { *self.noff.get() }
    }

    #[inline(always)]
    pub fn intena(&self) -> bool {
        unsafe { *self.intena.get() }
    }

    #[inline(always)]
    pub fn set_intena(&self, b: bool) {
        unsafe {
            *self.intena.get() = b;
        }
    }

    #[inline(always)]
    pub fn add_noff(&self, v: usize) {
        unsafe {
            *self.noff.get() += v;
        }
    }

    #[inline(always)]
    pub fn sub_noff(&self, v: usize) {
        unsafe {
            *self.noff.get() -= v;
        }
    }

    #[inline(always)]
    pub fn set_proc<'a>(&self, p: *mut Proc) {
        unsafe {
            *self.proc.get() = p;
        }
    }

    #[inline(always)]
    pub fn get_proc(&self) -> Option<&Proc> {
        unsafe {
            let p = *self.proc.get();
            if p == core::ptr::null_mut() {
                None
            } else {
                Some(&*p)
            }
        }
    }
}

// Saved registers for kernel context switches.
struct Context {
    ra: usize,
    sp: usize,

    // callee-saved
    s0: usize,
    s1: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    s9: usize,
    s10: usize,
    s11: usize,
}

impl Context {
    const fn new() -> Self {
        Context {
            ra: 0,
            sp: 0,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
        }
    }
}

enum ProcState {
    UNUSED,
    USED,
    SLEEPING,
    RUNNABLE,
    RUNNING,
    ZOMBIE,
}

// Per-process state
pub struct Proc {
    lock: SpinLock,

    // p.lock must be held when using these:
    // Process state
    state: ProcState,
    // If not None then, sleeping on chan
    chan: Option<Channel>,
    // If non-zero, have been killed
    killed: bool,
    // Exit status to be returned to parent's wait
    xstate: i32,
    // Process ID
    pid: u32,

    // wait_lock must be held when using this:
    // Parent process
    //struct proc *parent;

    // these are private to the process, so p->lock need not be held.
    // Virtual address of kernel stack
    kstack: usize,
    // Size of process memory (bytes)
    sz: usize,
    // User page table
    //pagetable_t pagetable;
    // data page for trampoline.S
    //struct trapframe *trapframe;
    // swtch() here to run process
    context: Context,
    //struct file *ofile[NOFILE];  // Open files
    //struct inode *cwd;           // Current directory
    // Process name (debugging)
    name: &'static str,
}

impl Cpu {
    const fn new() -> Self {
        Cpu {
            proc: UnsafeCell::new(core::ptr::null_mut()),
            context: Context::new(),
            noff: UnsafeCell::new(0),
            intena: UnsafeCell::new(false),
        }
    }
}

// Cpu is thread safe because each thread accesses it's own cpu from CPUS[].
unsafe impl Sync for Cpu {}

// Because all fields of Cpu are valid when zeroed, this should work.
static CPUS: [Cpu; param::NCPU] =
    unsafe { MaybeUninit::<[Cpu; param::NCPU]>::zeroed().assume_init() };

// Must be called with interrupts disabled,
// to prevent race with process being moved
// to a different CPU.
pub fn cpuid() -> usize {
    riscv::r_tp()
}

pub fn mycpu() -> &'static Cpu {
    &CPUS[cpuid()]
}

// Return the current struct proc reference, or None.
pub fn myproc<'a>() -> Option<&'a Proc> {
    SpinLock::push_off();
    let p = mycpu().get_proc();
    SpinLock::pop_off();
    p
}

pub fn sleep(chan: Channel, lock: &SpinLock) {
    unimplemented!("proc_sleep");
}

pub fn wakeup(chan: Channel) {
    unimplemented!("proc_wakeup");
}

pub fn either_copyin(dst: &mut [u8], user_src: bool, src: usize) {
    unimplemented!("proc_copyin");
}

pub fn proc_mapstacks(kpgtbl: &mut vm::PageTable) {
    unimplemented!("proc_mapstacks");
}
