//
// formatted console output -- printf, panic.
//

use super::console;
use super::spinlock::SpinLock;
use core::fmt::{self, Write};

pub static mut PANICKING: bool = false; // printing a panic message
pub static mut PANICKED: bool = false; // spinning forever at end of a panic

// lock to avoid interleaving concurrent printf's.
static PR: SpinLock = SpinLock::initlock("pr");

// Marker function to know when it's safe to call printf!.
pub fn printfinit() {}

#[macro_export]
macro_rules! printf {
    ($($arg:tt)*) => ($crate::kmain::printf::_print(format_args!($($arg)*)));
}

struct PrintfWriter;

impl Write for PrintfWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            console::consputc(b as u16)
        }
        Ok(())
    }
}

// Print to the console.
pub fn _print(args: fmt::Arguments) {
    unsafe {
        if PANICKING {
            PR.acquire();
        }
    }
    let mut writer = PrintfWriter;
    writer.write_fmt(args).unwrap();
    unsafe {
        if PANICKING {
            PR.release();
        }
    }
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        PANICKING = true;
    }
    printf!("\n--- KERNEL PANIC ---\n");

    // Print the location and message if available
    if let Some(location) = info.location() {
        printf!(
            "Location: {}:{}:{}\n",
            location.file(),
            location.line(),
            location.column()
        );
    }
    printf!("Message: {}\n", info.message());

    unsafe {
        // freeze uart output from other CPUs
        PANICKED = true;
    }
    // Halt the CPU
    loop {
        // On RISC-V, 'wfi' (Wait For Interrupt) saves power while looping
        unsafe { core::arch::asm!("wfi") };
    }
}
