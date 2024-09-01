#![no_std]
#![no_main]

use lib::{
    cstr16, println,
    println,
    uefi::{
        helper,
        protocols::{LoadedImageProtocol, Protocol, ProtocolLocateError},
        status::Status,
        Handle, SystemTable,
    },
};

#[panic_handler]
fn _panic_handler(panic_info: &core::panic::PanicInfo) -> ! {
    // NOTE: PanicInfo#payload isn't created in core, since it requires allocation.
    //
    if helper::_st_is_set() {
        println!("panic occurred: {:?}", panic_info);
        // FIXME: PanicInfo#message is getting stabilized in 1.81

        // if let Some(msg) = panic_info.message() {
        //     println!("panic occurred: {}", msg);
        // } else {
        //     println!("panic occurred");
        // }
    }
    loop {}
}

#[no_mangle]
pub extern "efiapi" fn efi_main(image_handle: Handle, mut system_table: SystemTable) -> Status {
    helper::register_services(&system_table);
    let boot_services = system_table.boot_services();

    println!("Hello, World!");

    let res = LoadedImageProtocol::try_locate(image_handle, &boot_services);
    let s = match res {
        Ok(_) => cstr16!("No error"),
        Err(ProtocolLocateError::Unsupported) => cstr16!("Unsupported protocol"),
        Err(ProtocolLocateError::Error(_)) => cstr16!("Other error"),
    };
    system_table.stdout().write(s);

    panic!("loop")
}
