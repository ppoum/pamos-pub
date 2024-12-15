# PARAMS
OVMF_PATH=/usr/share/edk2-ovmf
# =====

RUST_SRC=$(shell find ./src/ -name "*.rs") Cargo.toml

RELEASE_BIN_PATH=target/x86_64-unknown-uefi/release/pub.efi
DEBUG_BIN_PATH=target/x86_64-unknown-uefi/debug/pub.efi

.PHONY: .esp/kernel.bin .esp-dbg/kernel.bin

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

.esp/kernel.bin:
	cd ../pamos/ && make release
	cp ../pamos/target/x86_64-unknown-none/release/pamos $@

.esp-dbg/kernel.bin:
	cd ../pamos/ && make debug
	cp ../pamos/target/x86_64-unknown-none/debug/pamos $@

qemu: .esp/EFI/BOOT/BOOTX64.EFI .esp/kernel.bin
	qemu-system-x86_64 -d int,cpu_reset -enable-kvm -s -drive \
		if=pflash,format=raw,readonly=on,file=$(OVMF_PATH)/OVMF_CODE.fd \
		-drive \
		if=pflash,format=raw,readonly=on,file=$(OVMF_PATH)/OVMF_VARS.fd \
		-net none -drive file=fat:rw:.esp,format=raw

debug: .esp-dbg/EFI/BOOT/BOOTX64.EFI .esp-dbg/kernel.bin
	qemu-system-x86_64 -d int,cpu_reset -s -S -drive \
		if=pflash,format=raw,readonly=on,file=$(OVMF_PATH)/OVMF_CODE.fd \
		-drive \
		if=pflash,format=raw,readonly=on,file=$(OVMF_PATH)/OVMF_VARS.fd \
		-net none -drive file=fat:rw:.esp-dbg,format=raw

debug-nowait: .esp-dbg/EFI/BOOT/BOOTX64.EFI .esp-dbg/kernel.bin
	qemu-system-x86_64 -d int,cpu_reset -s -drive \
		if=pflash,format=raw,readonly=on,file=$(OVMF_PATH)/OVMF_CODE.fd \
		-drive \
		if=pflash,format=raw,readonly=on,file=$(OVMF_PATH)/OVMF_VARS.fd \
		-net none -drive file=fat:rw:.esp-dbg,format=raw
