/// Console input and output, to the uart.
/// Reads are one line at a time.
/// Implements special input characters:
///     newline -- end of line
///     control-h -- backspace
///     control-u -- kill line
///     control-d -- endof file
///     control-p -- print process list
use crate::drivers::uart;

// Erase the last output character.
const BACKSPACE: u16 = 0x100;

pub fn consoleinit() {
    uart::uartinit();
}

/// Send one character to the uart, but don't use interrupts or sleep. Safe to
/// be called from interrupts, e.g. by printf and to echo input characters.
pub fn putc(c: u16) {
    if c == BACKSPACE {
        // If the user typed backspace, overwrite with a space.
        let ascii_backspace = 0x08u8;
        uart::putc_sync(ascii_backspace);
        uart::putc_sync(b' ');
        uart::putc_sync(ascii_backspace);
    } else {
        uart::putc_sync(c as u8);
    }
}
