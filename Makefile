# PARAMS
OVMF_PATH=/usr/share/edk2-ovmf
# =====

RUST_SRC=$(shell find ./src/ -name "*.rs") Cargo.toml

RELEASE_BIN_PATH=target/x86_64-unknown-uefi/release/pub.efi
DEBUG_BIN_PATH=target/x86_64-unknown-uefi/debug/pub.efi

$(RELEASE_BIN_PATH): $(RUST_SRC)
	cargo b --release

img: uefi.img

uefi.img: $(RELEASE_BIN_PATH)
	dd if=/dev/zero of=$@ bs=1k count=1440
	mformat -i $@ -f 1440 ::
	mmd -i $@ ::/EFI
	mmd -i $@ ::/EFI/BOOT
	mcopy -i $@ $< ::/EFI/BOOT

qemu: uefi.img
	qemu-system-x86_64 -drive \
		if=pflash,format=raw,readonly=on,file=$(OVMF_PATH)/OVMF_CODE.fd \
		-drive \
		if=pflash,format=raw,readonly=on,file=$(OVMF_PATH)/OVMF_VARS.fd \
		-net none -cdrom uefi.img
