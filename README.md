# PUB (Pam Unified Bootloader)

It's GRUB but worse

## Build dependencies

- Rust (with x86_64-unknown-uefi target)
- OVMF binaries (only required if running the qemu make targets)
- [pamos](https://github.com/ppoum/pamos)
  - For the `Makefile` to work properly, the repository for `pamos` should be
    in the same parent directory and named `pamos` (ie: `../pamos/`)
