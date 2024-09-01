#![no_std]
#![no_main]

use lib::{
    println,
    uefi::{
        helper,
        protocols::{LoadedImageProtocol, Protocol, ProtocolLocateError, SimpleFileSystemProtocol},
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

    1.into()
}
