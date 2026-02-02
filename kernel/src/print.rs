/// Formatted console output functions.
use crate::arch::{Arch, CurrentArch};
use crate::kstate;

/// Print to the console.
pub macro print($($arg:tt)*) {
    _print(format_args!($($arg)*));
}

/// Print to the console, with a newline.
pub macro println($($arg:tt)*) {
    print!("{}\n", format_args!($($arg)*));
}

pub fn _print(args: core::fmt::Arguments) {
    use crate::console::putc;
    use core::fmt::Write;

    struct ConsoleWriter;

    impl Write for ConsoleWriter {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            for byte in s.bytes() {
                putc(byte as u16);
            }
            Ok(())
        }
    }

    let mut writer = ConsoleWriter;
    let _ = if kstate::is_panicking() {
        // If the kernel is panicking, acquire lock to print panic message.
        PRINT_LOCK.with_lock(|| writer.write_fmt(args))
    } else {
        writer.write_fmt(args)
    };
}

static PRINT_LOCK: crate::spinlock::Spinlock = crate::spinlock::Spinlock::new("print_lock");

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    kstate::set_panicking();
    println!("\n--- KERNEL PANIC ---\n");

    // Print the location and message if available
    if let Some(location) = info.location() {
        println!(
            "Location: {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }
    println!("Message: {}", info.message());

    // Freeze output from other CPUs
    kstate::set_panicked();
    // Halt the CPU
    loop {
        CurrentArch::halt();
    }
}
