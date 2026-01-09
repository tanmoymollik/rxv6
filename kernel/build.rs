// use cc;
// use std::{error, fs};
use std::error;

// const ASM_DIR: &str = "src/asm";

fn main() -> Result<(), Box<dyn error::Error>> {
    // let mut compiler = cc::Build::new();

    // for entry in fs::read_dir(ASM_DIR)? {
    //     let path = entry?.path();
    //     // Force a rebuild if the asm file changes.
    //     println!("cargo:rerun-if-changed={}", path.display());
    //     compiler.file(path);
    // }

    // compiler
    //     .flag("-march=rv64gc")
    //     .flag("-mabi=lp64d")
    //     .compile("entry");
    println!("cargo:rustc-link-arg=-Tkernel/kernel.ld");
    // Set max-page-size = 4K for riscv64.
    println!("cargo:rustc-link-arg=-z");
    println!("cargo:rustc-link-arg=max-page-size=4096");

    // Force a rebuild if the linker script or asm folder changes.
    println!("cargo:rerun-if-changed=kernel.ld");
    //println!("cargo:rerun-if-changed={ASM_DIR}");
    Ok(())
}
