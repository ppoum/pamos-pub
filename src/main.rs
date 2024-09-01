#![no_std]
#![no_main]

use lib::{
    cstr16,
    elf::{Elf64Ehdr, ElfClass, ElfDataLayout, ElfMachine, ElfType},
    println,
    uefi::{
        helper,
        protocols::{
            FileAttribute, FileMode, FileProtocol, LoadedImageProtocol, Protocol,
            ProtocolLocateError, SimpleFileSystemProtocol,
        },
        status::Status,
        Handle, SystemTable,
    },
};

// Helper function for now
fn unwrap_protocol_result<T>(res: Result<T, ProtocolLocateError>) -> T {
    match res {
        Ok(p) => return p,
        Err(ProtocolLocateError::Unsupported) => println!("Unsupported protocol"),
        Err(ProtocolLocateError::Error(_)) => println!("Other error"),
    };
    panic!()
}

fn validate_kernel_elf(file: &FileProtocol) -> bool {
    // Read the ELF header
    let mut header: Elf64Ehdr = Default::default();
    match file.read(&mut header) {
        Ok(true) => println!("Read the ELF header in full"),
        Ok(false) => println!("WARN: Partially read the elf header"),
        Err(e) => {
            println!("ERR: error reading kernel binary: {:?}", e);
            return false;
        }
    };

    if !header.valid_magic() {
        println!("ERR: invalid ELF magic");
        return false;
    }

    if header.class() != ElfClass::Class64 {
        println!("ERR: kernel is not 64-bit");
        return false;
    }

    if header.data_layout() != ElfDataLayout::Lsb {
        println!("ERR: kernel doesn't use LSB data ordering");
        return false;
    }

    if header.elf_type() != ElfType::Executable && header.elf_type() != ElfType::Dynamic {
        println!("ERR: kernel ELF file isn't an executable file (ET_EXEC or ET_DYN)");
        return false;
    }

    if header.machine() != ElfMachine::X86_64 {
        println!("ERR: kernel is not a x86_64 binary");
        return false;
    }

    true
}

#[no_mangle]
pub extern "efiapi" fn efi_main(image_handle: Handle, mut system_table: SystemTable) -> Status {
    helper::register_services(&system_table);
    let boot_services = system_table.boot_services();

    println!("Hello, World!");

    let res = LoadedImageProtocol::try_locate(image_handle, &boot_services);
    let loaded_image = unwrap_protocol_result(res);

    // Get volume from our EFI app handle and open root path
    let res = SimpleFileSystemProtocol::try_locate(loaded_image.device(), &boot_services);
    let res = unwrap_protocol_result(res);
    let root = res.open_volume().expect("error opening root volume");

    // Open the kernel file
    let kernel_file = root
        .open(
            cstr16!("kernel.bin"),
            FileMode::Read,
            FileAttribute::default(),
        )
        .expect("Error opening kernel.bin file");
    println!("Opened the kernel.bin file");

    if !validate_kernel_elf(kernel_file) {
        panic!("Kernel validation failed");
    }
    println!("Kernel file validated");

    loop {}
}
