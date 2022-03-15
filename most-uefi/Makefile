MODE ?= release
EFI := ../target/x86_64-unknown-uefi/$(MODE)/most-uefi.efi
OVMF := OVMF.fd
ESP := ../target/esp
QEMU_ARGS := -nographic
#	-debugcon file:debug.log -global isa-debugcon.iobase=0x402
OBJDUMP := rust-objdump

ifeq (${MODE}, release)
	BUILD_ARGS += --release
endif

.PHONY: build run header asm

build:
	cargo build $(BUILD_ARGS)
	mkdir -p $(ESP)/EFI/Boot
	cp $(EFI) $(ESP)/EFI/Boot/BootX64.efi

clippy:
	cargo clippy $(BUILD_ARGS)

run: build
	qemu-system-x86_64 \
		-bios ${OVMF} \
		-drive format=raw,file=fat:rw:${ESP} \
		$(QEMU_ARGS)

header:
	$(OBJDUMP) -h $(EFI) | less

asm:
	$(OBJDUMP) -d $(EFI) | less
