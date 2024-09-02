#![no_std]
#![no_main]

mod loader;

use lib::{
    cstr16, println,
    uefi::{
        helper::{self},
        protocols::{
            FileAttribute, FileMode, LoadedImageProtocol, Protocol, ProtocolLocateError,
            SimpleFileSystemProtocol,
        },
        status::Status,
        Handle, SystemTable,
    },
};
use loader::ElfKernel;

// Helper function for now
fn unwrap_protocol_result<T>(res: Result<T, ProtocolLocateError>) -> T {
    match res {
        Ok(p) => return p,
        Err(ProtocolLocateError::Unsupported) => println!("Unsupported protocol"),
        Err(ProtocolLocateError::Error(_)) => println!("Other error"),
    };
    panic!()
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

    let _kernel = ElfKernel::load_from_file(kernel_file, system_table.boot_services())
        .expect("error reading kernel file");

    println!("Kernel file loaded");

    // let exit_code = unsafe { kernel.entrypoint()() };
    // println!("Kernel exited with code: {}", exit_code);

    loop {}
}
