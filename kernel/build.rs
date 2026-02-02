use std::env::var;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // build.rs is run from the workspace root.
    println!("cargo:rustc-link-arg=-Tkernel/kernel.ld");
    // Define number of cpus for the link script.
    let ncpu = var("NCPU").unwrap_or("1".into());
    ncpu.parse::<usize>().expect("NCPU must be a number");
    println!("cargo:rustc-link-arg=--defsym=NCPU={ncpu}");
    // Set max-page-size = 4K for riscv64.
    println!("cargo:rustc-link-arg=-z");
    println!("cargo:rustc-link-arg=max-page-size=4096");
    // Force a rebuild if the linker script changes.
    println!("cargo:rerun-if-changed=kernel.ld");
    println!("cargo:rerun-if-env-changed=NCPU");
    Ok(())
}
