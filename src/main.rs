#![no_std]
#![no_main]

use lib::{
    cstr16,
    uefi::{
        protocols::{LoadedImageProtocol, Protocol, ProtocolLocateError},
        status::Status,
        Handle, SystemTable,
    },
};

#[panic_handler]
fn _panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "efiapi" fn efi_main(image_handle: Handle, mut system_table: SystemTable) -> Status {
    let boot_services = system_table.boot_services();

    let s = cstr16!("Hello, World!\n");
    system_table.stdout().write(s);

    let res = LoadedImageProtocol::try_locate(image_handle, &boot_services);
    let s = match res {
        Ok(_) => cstr16!("No error"),
        Err(ProtocolLocateError::Unsupported) => cstr16!("Unsupported protocol"),
        Err(ProtocolLocateError::Error(_)) => cstr16!("Other error"),
    };
    system_table.stdout().write(s);

    panic!("loop")
}
