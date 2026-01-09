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
// #include "types.h"
// #include "param.h"
// #include "spinlock.h"
// #include "sleeplock.h"
// #include "fs.h"
// #include "file.h"
// #include "memlayout.h"
// #include "riscv.h"
// #include "defs.h"
// #include "proc.h"

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
    //   uint target;
    //   int c;
    //   char cbuf;

    //   target = n;
    //   acquire(&cons.lock);
    //   while(n > 0){
    //     // wait until interrupt handler has put some
    //     // input into cons.buffer.
    //     while(cons.r == cons.w){
    //       if(killed(myproc())){
    //         release(&cons.lock);
    //         return -1;
    //       }
    //       sleep(&cons.r, &cons.lock);
    //     }

    //     c = cons.buf[cons.r++ % INPUT_BUF_SIZE];

    //     if(c == C('D')){  // end-of-file
    //       if(n < target){
    //         // Save ^D for next time, to make sure
    //         // caller gets a 0-byte result.
    //         cons.r--;
    //       }
    //       break;
    //     }

    //     // copy the input byte to the user-space buffer.
    //     cbuf = c;
    //     if(either_copyout(user_dst, dst, &cbuf, 1) == -1)
    //       break;

    //     dst++;
    //     --n;

    //     if(c == '\n'){
    //       // a whole line has arrived, return to
    //       // the user-level read().
    //       break;
    //     }
    //   }
    //   release(&cons.lock);

    //   return target - n;
}

//
// the console input interrupt handler.
// uartintr() calls this for each input character.
// do erase/kill processing, append to cons.buf,
// wake up consoleread() if a whole line has arrived.
//
pub fn consoleintr(c: u8) {
    //   acquire(&cons.lock);

    //   switch(c){
    //   case C('P'):  // Print process list.
    //     procdump();
    //     break;
    //   case C('U'):  // Kill line.
    //     while(cons.e != cons.w &&
    //           cons.buf[(cons.e-1) % INPUT_BUF_SIZE] != '\n'){
    //       cons.e--;
    //       consputc(BACKSPACE);
    //     }
    //     break;
    //   case C('H'): // Backspace
    //   case '\x7f': // Delete key
    //     if(cons.e != cons.w){
    //       cons.e--;
    //       consputc(BACKSPACE);
    //     }
    //     break;
    //   default:
    //     if(c != 0 && cons.e-cons.r < INPUT_BUF_SIZE){
    //       c = (c == '\r') ? '\n' : c;

    //       // echo back to the user.
    //       consputc(c);

    //       // store for consumption by consoleread().
    //       cons.buf[cons.e++ % INPUT_BUF_SIZE] = c;

    //       if(c == '\n' || c == C('D') || cons.e-cons.r == INPUT_BUF_SIZE){
    //         // wake up consoleread() if a whole line (or end-of-file)
    //         // has arrived.
    //         cons.w = cons.e;
    //         wakeup(&cons.r);
    //       }
    //     }
    //     break;
    //   }

    //   release(&cons.lock);
}

pub fn consoleinit() {
    uart::uartinit();

    // connect read and write system calls
    // to consoleread and consolewrite.
    //   devsw[CONSOLE].read = consoleread;
    //   devsw[CONSOLE].write = consolewrite;
}
