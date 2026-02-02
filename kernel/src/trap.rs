use crate::arch::{Arch, CurrentArch};

core::arch::global_asm!(include_str!("asm/kernelvec.S"), kerneltrap = sym kerneltrap);

pub fn trapinithart() {
    CurrentArch::set_trap_vector(kerneltrap);
}

fn kerneltrap() {}
