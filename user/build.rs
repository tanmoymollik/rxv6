use std::env::var;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // build.rs is run from the workspace root.
    println!("cargo:rustc-link-arg=-Tuser/linker.ld");
    // Set max-page-size = 4K for riscv64.
    println!("cargo:rustc-link-arg=-z");
    println!("cargo:rustc-link-arg=max-page-size=4096");
    // Force a rebuild if the linker script changes.
    println!("cargo:rerun-if-changed=linker.ld");
    Ok(())
}
