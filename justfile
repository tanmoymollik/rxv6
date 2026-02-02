TARGET := "riscv64gc-unknown-none-elf"
KERNEL_BIN := f"target/{{TARGET}}/debug/kernel"
OUT_DIR := f"target/{{TARGET}}/debug"

# When changing NCPU, also change in kernel/src/param.rs.

NCPU := "4"
QEMU := "qemu-system-riscv64"
DBGPORT := "1234"
QEMUOPTS := ("-machine virt " + "-bios none " + f"-kernel {{KERNEL_BIN}} " + "-m 128M " + f"-smp {{NCPU}} " + "-nographic " + "-global virtio-mmio.force-legacy=false " + "-drive file=fs.img,if=none,format=raw,id=x0 " + "-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0")
QEMUDBG := f"-gdb tcp::{{DBGPORT}} -S"

clean:
    cargo clean
    rm -f .lldbinit UserManifest.toml fs.img dump

build-kernel:
    NCPU={{ NCPU }} cargo build -p kernel --target {{ TARGET }}

build-user:
    cargo build -p user --target {{ TARGET }}
    awk '{ print $1 " = \"{{ OUT_DIR }}/" $1 "\"" }' user/UserManifest > UserManifest.toml

mkfs: build-user
    cargo run -p mkfs -- fs.img UserManifest.toml

build: build-kernel build-user mkfs

objdump: build-kernel
    riscv64-unknown-elf-objdump -d {{ KERNEL_BIN }} > dump

run: build
    {{ QEMU }} {{ QEMUOPTS }}

lldbinit:
    echo "target create {{ KERNEL_BIN }}" > .lldbinit
    echo "gdb-remote {{ DBGPORT }}" >> .lldbinit

dbg-run: build
    {{ QEMU }} {{ QEMUOPTS }} {{ QEMUDBG }}
