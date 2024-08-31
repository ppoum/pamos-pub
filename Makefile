# PARAMS
OVMF_PATH=/usr/share/edk2-ovmf
# =====

RUST_SRC=$(shell find ./src/ -name "*.rs") Cargo.toml

RELEASE_BIN_PATH=target/x86_64-unknown-uefi/release/pub.efi
DEBUG_BIN_PATH=target/x86_64-unknown-uefi/debug/pub.efi

$(RELEASE_BIN_PATH): $(RUST_SRC)
	cargo b --release
$(DEBUG_BIN_PATH): $(RUST_SRC)
	cargo b

.esp-dbg/EFI/BOOT/BOOTX64.EFI: $(DEBUG_BIN_PATH)
	mkdir -p $$(dirname $@)
	cp $< $@
.esp/EFI/BOOT/BOOTX64.EFI: $(RELEASE_BIN_PATH)
	mkdir -p $$(dirname $@)
	cp $< $@

qemu: .esp/EFI/BOOT/BOOTX64.EFI
	qemu-system-x86_64 -enable-kvm -s -drive \
		if=pflash,format=raw,readonly=on,file=$(OVMF_PATH)/OVMF_CODE.fd \
		-drive \
		if=pflash,format=raw,readonly=on,file=$(OVMF_PATH)/OVMF_VARS.fd \
		-net none -drive file=fat:rw:.esp,format=raw

debug: .esp-dbg/EFI/BOOT/BOOTX64.EFI
	qemu-system-x86_64 -enable-kvm -s -S -drive \
		if=pflash,format=raw,readonly=on,file=$(OVMF_PATH)/OVMF_CODE.fd \
		-drive \
		if=pflash,format=raw,readonly=on,file=$(OVMF_PATH)/OVMF_VARS.fd \
		-net none -drive file=fat:rw:.esp-dbg,format=raw
