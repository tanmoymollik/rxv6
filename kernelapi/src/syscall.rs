/// System call numbers.
enum Syscall {
    SysFork = 1,
    SysExit = 2,
    SysWait = 3,
    SysPipe = 4,
    SysRead = 5,
    SysKill = 6,
    SysExec = 7,
    SysFstat = 8,
    SysChdir = 9,
    SysDup = 10,
    SysGetpid = 11,
    SysSbrk = 12,
    SysPause = 13,
    SysUptime = 14,
    SysOpen = 15,
    SysWrite = 16,
    SysMknod = 17,
    SysUnlink = 18,
    SysLink = 19,
    SysMkdir = 20,
    SysClose = 21,
}

pub fn write(_: i32) {
    if cfg!(target_arch = "riscv64") {
        unsafe {
            core::arch::asm!("ecall");
        }
    }
}
