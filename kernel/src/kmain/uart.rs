//
// low-level driver for 16550a UART.
//

use super::channel::Channel;
use super::console;
use super::printf;
use super::proc;
use super::spinlock::SpinLock;
use crate::memlayout;
use core::sync::atomic::{AtomicBool, Ordering};

// the UART control registers are memory-mapped
// at address UART0. this macro returns the
// address of one of the registers.
#[inline(always)]
fn reg(reg: u8) -> *mut u8 {
    (memlayout::UART0 + reg as usize) as *mut u8
}

#[inline(always)]
fn read_reg(r: u8) -> u8 {
    unsafe { core::ptr::read_volatile(reg(r)) }
}

#[inline(always)]
fn write_reg(r: u8, v: u8) {
    unsafe { core::ptr::write_volatile(reg(r), v) }
}

// the UART control registers.
// some have different meanings for read vs write.
// see http://byterunner.com/16550.html
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

// for sending threads to synchronize with uart "ready" interrupts.
static TX_LOCK: SpinLock = SpinLock::initlock("uart");
static TX_BUSY: AtomicBool = AtomicBool::new(false); // is the UART busy sending?

pub fn uartinit() {
    // disable interrupts.
    write_reg(IER, 0x00);

    // special mode to set baud rate.
    write_reg(LCR, LCR_BAUD_LATCH);

    // LSB for baud rate of 38.4K.
    write_reg(0, 0x03);

    // MSB for baud rate of 38.4K.
    write_reg(1, 0x00);

    // leave set-baud mode,
    // and set word length to 8 bits, no parity.
    write_reg(LCR, LCR_EIGHT_BITS);

    // reset and enable FIFOs.
    write_reg(FCR, FCR_FIFO_ENABLE | FCR_FIFO_CLEAR);

    // enable transmit and receive interrupts.
    write_reg(IER, IER_TX_ENABLE | IER_RX_ENABLE);
}

// transmit buf[] to the uart. it blocks if the
// uart is busy, so it cannot be called from
// interrupts, only from write() system calls.
fn uartwrite(buf: &[u8], n: usize) {
    TX_LOCK.acquire();

    for i in (0..n) {
        while TX_BUSY.load(Ordering::Relaxed) {
            // wait for a UART transmit-complete interrupt
            // to set tx_busy to 0.
            proc::sleep(Channel::Uart, &TX_LOCK);
        }

        write_reg(THR, buf[i]);
        TX_BUSY.store(true, Ordering::Relaxed);
    }

    TX_LOCK.release();
}

// write a byte to the uart without using
// interrupts, for use by kernel printf() and
// to echo characters. it spins waiting for the uart's
// output register to be empty.
pub fn uartputc_sync(c: u8) {
    let panicking = unsafe { printf::PANICKING };
    let panicked = unsafe { printf::PANICKED };
    if !panicking {
        SpinLock::push_off();
    }

    if panicked {
        loop {}
    }

    // wait for UART to set Transmit Holding Empty in LSR.
    while (read_reg(LSR) & LSR_TX_IDLE) == 0 {}
    write_reg(THR, c);

    if !panicking {
        SpinLock::pop_off();
    }
}

// try to read one input character from the UART.
// return -1 if none is waiting.
fn uartgetc() -> Option<u8> {
    if read_reg(LSR) & LSR_RX_READY > 0 {
        // input data is ready.
        return Some(read_reg(RHR));
    } else {
        return None;
    }
}

// handle a uart interrupt, raised because input has
// arrived, or the uart is ready for more output, or
// both. called from devintr().
fn uartintr() {
    // acknowledge the interrupt
    read_reg(ISR);

    TX_LOCK.acquire();
    if read_reg(LSR) & LSR_TX_IDLE > 0 {
        // UART finished transmitting; wake up sending thread.
        TX_BUSY.store(false, Ordering::Relaxed);
        proc::wakeup(Channel::Uart);
    }
    TX_LOCK.release();

    // read and process incoming characters, if any.
    loop {
        match uartgetc() {
            None => break,
            Some(c) => console::consoleintr(c),
        }
    }
}
