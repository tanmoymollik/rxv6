#![no_std]
#![no_main]

#[unsafe(no_mangle)]
fn main() {
    let x = 2 + 2;
    if cfg!(target_arch = "riscv64") {
        kernelapi::syscall::write(x);
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
