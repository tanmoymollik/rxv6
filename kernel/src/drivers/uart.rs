// Low-level driver for 16550a UART.
use crate::kstate;
use crate::kutils::without_interrupts;
use crate::memlayout;

pub fn uartinit() {
    // Disable interrupts.
    write_reg(IER, 0x00);

    // Special mode to set baud rate.
    write_reg(LCR, LCR_BAUD_LATCH);

    // LSB for baud rate of 38.4K.
    write_reg(0, 0x03);

    // MSB for baud rate of 38.4K.
    write_reg(1, 0x00);

    // Leave set-baud mode and set word length to 8 bits, no parity.
    write_reg(LCR, LCR_EIGHT_BITS);

    // Reset and enable FIFOs.
    write_reg(FCR, FCR_FIFO_ENABLE | FCR_FIFO_CLEAR);

    // Enable transmit and receive interrupts.
    write_reg(IER, IER_TX_ENABLE | IER_RX_ENABLE);
}

/// Write a byte to the UART without using interrupts, for use by kernel
/// printf() and to echo characters. It spins waiting for the UART's output
/// register to be empty.
pub fn putc_sync(c: u8) {
    let transmit = || {
        // Wait for UART to set Transmit Holding Empty in LSR.
        while (read_reg(LSR) & LSR_TX_IDLE) == 0 {}
        write_reg(THR, c);
    };
    let is_panicking = kstate::is_panicking();
    let has_panicked = kstate::has_panicked();

    if has_panicked {
        // If the kernel has already panicked, disable all cpu output.
        loop {}
    }
    if is_panicking {
        // If we are panicking, just run the closure without disabling
        // interrupts because the cpu is in an inconsistent state.
        transmit();
    } else {
        without_interrupts(transmit);
    }
}

// The UART control registers are memory-mapped at address `memlayout::UART0`.
// This macro returns the address of one of the registers.
#[inline(always)]
fn reg(reg: u8) -> usize {
    memlayout::UART0 + reg as usize
}

#[inline(always)]
fn read_reg(r: u8) -> u8 {
    unsafe { (reg(r) as *const u8).read_volatile() }
}

#[inline(always)]
fn write_reg(r: u8, v: u8) {
    unsafe {
        (reg(r) as *mut u8).write_volatile(v);
    }
}

// The UART control registers.
// Some have different meanings for read vs write.
// See http://byterunner.com/16550.html
const RHR: u8 = 0; // receive holding register (for input bytes)
const THR: u8 = 0; // transmit holding register (for output bytes)
const IER: u8 = 1; // interrupt enable register
const IER_RX_ENABLE: u8 = 1 << 0;
const IER_TX_ENABLE: u8 = 1 << 1;
const FCR: u8 = 2; // FIFO control register
const FCR_FIFO_ENABLE: u8 = 1 << 0;
const FCR_FIFO_CLEAR: u8 = 3 << 1; // clear the content of the two FIFOs
const ISR: u8 = 2; // interrupt status register
const LCR: u8 = 3; // line control register
const LCR_EIGHT_BITS: u8 = 3 << 0;
const LCR_BAUD_LATCH: u8 = 1 << 7; // special mode to set baud rate
const LSR: u8 = 5; // line status register
const LSR_RX_READY: u8 = 1 << 0; // input is waiting to be read from RHR
const LSR_TX_IDLE: u8 = 1 << 5; // THR can accept another character to send
