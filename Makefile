.PHONY: kernel clean qemu qemu-gdb qemu-lldb symbols

K=kernel
U=user

# riscv64-unknown-elf- or riscv64-linux-gnu-
# perhaps in /opt/riscv/bin

# Try to infer the correct TOOLPREFIX if not set
ifndef TOOLPREFIX
TOOLPREFIX := $(shell if riscv64-unknown-elf-objdump -i 2>&1 | grep 'elf64-big' >/dev/null 2>&1; \
	then echo 'riscv64-unknown-elf-'; \
	elif riscv64-elf-objdump -i 2>&1 | grep 'elf64-big' >/dev/null 2>&1; \
	then echo 'riscv64-elf-'; \
	elif riscv64-linux-gnu-objdump -i 2>&1 | grep 'elf64-big' >/dev/null 2>&1; \
	then echo 'riscv64-linux-gnu-'; \
	elif riscv64-unknown-linux-gnu-objdump -i 2>&1 | grep 'elf64-big' >/dev/null 2>&1; \
	then echo 'riscv64-unknown-linux-gnu-'; \
	else echo "***" 1>&2; \
	echo "*** Error: Couldn't find a riscv64 version of GCC/binutils." 1>&2; \
	echo "*** To turn off this error, run 'gmake TOOLPREFIX= ...'." 1>&2; \
	echo "***" 1>&2; exit 1; fi)
endif

QEMU = qemu-system-riscv64
MIN_QEMU_VERSION = 7.2

CC = $(TOOLPREFIX)gcc
AS = $(TOOLPREFIX)gas
LD = $(TOOLPREFIX)ld
OBJCOPY = $(TOOLPREFIX)objcopy
OBJDUMP = $(TOOLPREFIX)objdump
OUT_DIR = target/riscv64gc-unknown-none-elf/debug

dump:
	mkdir dump

kernel: dump
	cargo build -p kernel

symbols: $(OUT_DIR)/kernel
	$(OBJDUMP) -S $(OUT_DIR)/kernel > dump/kernel.asm
	$(OBJDUMP) -t $(OUT_DIR)/kernel | sed '1,/SYMBOL TABLE/d; s/ .* / /; /^$$/d' > dump/kernel.sym

# ULIB = $U/ulib.o $U/usys.o $U/printf.o $U/umalloc.o

# _%: %.o $(ULIB) $U/user.ld
# 	$(LD) $(LDFLAGS) -T $U/user.ld -o $@ $< $(ULIB)
# 	$(OBJDUMP) -S $@ > $*.asm
# 	$(OBJDUMP) -t $@ | sed '1,/SYMBOL TABLE/d; s/ .* / /; /^$$/d' > $*.sym

# $U/usys.S : $U/usys.pl
# 	perl $U/usys.pl > $U/usys.S

# $U/usys.o : $U/usys.S
# 	$(CC) $(CFLAGS) -c -o $U/usys.o $U/usys.S

# $U/_forktest: $U/forktest.o $(ULIB)
# 	# forktest has less library code linked in - needs to be small
# 	# in order to be able to max out the proc table.
# 	$(LD) $(LDFLAGS) -N -e main -Ttext 0 -o $U/_forktest $U/forktest.o $U/ulib.o $U/usys.o
# 	$(OBJDUMP) -S $U/_forktest > $U/forktest.asm

# mkfs/mkfs: mkfs/mkfs.c $K/fs.h $K/param.h
# 	gcc -Wno-unknown-attributes -I. -o mkfs/mkfs mkfs/mkfs.c

# Prevent deletion of intermediate files, e.g. cat.o, after first build, so
# that disk image changes after first build are persistent until clean.  More
# details:
# http://www.gnu.org/software/make/manual/html_node/Chained-Rules.html
# .PRECIOUS: %.o

# UPROGS=\
# 	$U/_cat\
# 	$U/_echo\
# 	$U/_forktest\
# 	$U/_grep\
# 	$U/_init\
# 	$U/_kill\
# 	$U/_ln\
# 	$U/_ls\
# 	$U/_mkdir\
# 	$U/_rm\
# 	$U/_sh\
# 	$U/_stressfs\
# 	$U/_usertests\
# 	$U/_grind\
# 	$U/_wc\
# 	$U/_zombie\
# 	$U/_logstress\
# 	$U/_forphan\
# 	$U/_dorphan\

# fs.img: mkfs/mkfs README $(UPROGS)
# 	mkfs/mkfs fs.img README $(UPROGS)

-include kernel/*.d user/*.d

clean: 
	cargo clean
	rm -rf dump
	rm -f .gdbinit
# 	rm -f *.tex *.dvi *.idx *.aux *.log *.ind *.ilg \
# 	*/*.o */*.d */*.asm */*.sym \
# 	$K/kernel fs.img \
# 	mkfs/mkfs .gdbinit \
#         $U/usys.S \
# 	$(UPROGS)

# try to generate a unique GDB port
GDBPORT = $(shell expr `id -u` % 5000 + 25000)
# QEMU's gdb stub command line changed in 0.11
QEMUGDB = $(shell if $(QEMU) -help | grep -q '^-gdb'; \
	then echo "-gdb tcp::$(GDBPORT)"; \
	else echo "-s -p $(GDBPORT)"; fi)
ifndef CPUS
CPUS := 3
endif

QEMUOPTS = -machine virt -bios none -kernel $(OUT_DIR)/kernel -m 128M -smp $(CPUS) -nographic
QEMUOPTS += -global virtio-mmio.force-legacy=false
# QEMUOPTS += -drive file=fs.img,if=none,format=raw,id=x0
# QEMUOPTS += -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

# qemu: check-qemu-version $(OUT_DIR)/kernel fs.img
# 	$(QEMU) $(QEMUOPTS)
qemu: check-qemu-version kernel
	$(QEMU) $(QEMUOPTS)

.gdbinit: .gdbinit.tmpl-riscv
	sed "s/:1234/:$(GDBPORT)/" < $^ | \
	sed "s|out_dir|$(OUT_DIR)|" > $@

# qemu-gdb: $(OUT_DIR)/kernel .gdbinit fs.img
# 	@echo "*** Now run 'gdb' in another window." 1>&2
# 	$(QEMU) $(QEMUOPTS) -S $(QEMUGDB)
qemu-gdb: kernel .gdbinit
	@echo "*** Now run 'gdb' in another window." 1>&2
	$(QEMU) $(QEMUOPTS) -S $(QEMUGDB)

.lldbinit: .lldbinit.tmpl-riscv
	sed "s/:1234/:$(GDBPORT)/" < $^ | \
	sed "s|out_dir|$(OUT_DIR)|" > $@

qemu-lldb: kernel .lldbinit
	@echo "*** Now run 'lldb' in another window." 1>&2
	$(QEMU) $(QEMUOPTS) -S $(QEMUGDB)

print-gdbport:
	@echo $(GDBPORT)

QEMU_VERSION := $(shell $(QEMU) --version | head -n 1 | sed -E 's/^QEMU emulator version ([0-9]+\.[0-9]+)\..*/\1/')
check-qemu-version:
	@if [ "$(shell echo "$(QEMU_VERSION) >= $(MIN_QEMU_VERSION)" | bc)" -eq 0 ]; then \
		echo "ERROR: Need qemu version >= $(MIN_QEMU_VERSION)"; \
		exit 1; \
	fi