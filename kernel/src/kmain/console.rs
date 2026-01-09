//
// Console input and output, to the uart.
// Reads are line at a time.
// Implements special input characters:
//   newline -- end of line
//   control-h -- backspace
//   control-u -- kill line
//   control-d -- endof file
//   control-p -- print process list
//

use super::proc;
use super::spinlock::SpinLock;
use super::uart;

const BACKSPACE: u16 = 0x100; // erase the last output character

// Control-x
#[inline(always)]
fn c(x: u8) -> u8 {
    x - '@' as u8
}

//
// send one character to the uart, but don't use
// interrupts or sleep(). safe to be called from
// interrupts, e.g. by printf and to echo input
// characters.
//
pub fn consputc(c: u16) {
    if c == BACKSPACE {
        // if the user typed backspace, overwrite with a space.
        let ascii_backspace = 0x08u8;
        uart::uartputc_sync(ascii_backspace);
        uart::uartputc_sync(b' ');
        uart::uartputc_sync(ascii_backspace);
    } else {
        uart::uartputc_sync(c as u8);
    }
}

const INPUT_BUF_SIZE: usize = 128;
struct Cons {
    lock: SpinLock,

    // input circular buffer
    buf: [u8; INPUT_BUF_SIZE],
    r: u32, // Read index
    w: u32, // Write index
    e: u32, // Edit index
}

// Cons is protected by a SpinLock
unsafe impl Sync for Cons {}

static CONS: Cons = Cons {
    lock: SpinLock::initlock("cons"),
    buf: [0; INPUT_BUF_SIZE],
    r: 0,
    w: 0,
    e: 0,
};

//
// user write() system calls to the console go here.
// uses sleep() and UART interrupts.
//
fn consolewrite(user_src: bool, src: usize, n: usize) {
    unimplemented!("console_consolewrite");
    //   char buf[32]; // move batches from user space to uart.
    //   int i = 0;

    //   while(i < n){
    //     int nn = sizeof(buf);
    //     if(nn > n - i)
    //       nn = n - i;
    //     if(proc::either_copyin(buf, user_src, src+i, nn) == -1)
    //       break;
    //     uartwrite(buf, nn);
    //     i += nn;
    //   }

    //   return i;
}

//
// user read()s from the console go here.
// copy (up to) a whole input line to dst.
// user_dst indicates whether dst is a user
// or kernel address.
//
fn consoleread(user_dst: bool, dst: usize, n: usize) {
    unimplemented!("console_consoleread");
}

//
// the console input interrupt handler.
// uartintr() calls this for each input character.
// do erase/kill processing, append to cons.buf,
// wake up consoleread() if a whole line has arrived.
//
pub fn consoleintr(c: u8) {
    unimplemented!("console_consoleintr");
}

pub fn consoleinit() {
    uart::uartinit();

    // connect read and write system calls
    // to consoleread and consolewrite.
    //   devsw[CONSOLE].read = consoleread;
    //   devsw[CONSOLE].write = consolewrite;
}
