#![no_std]
#![no_main]

use lib::{
    cstr16,
    uefi::{Handle, Status, SystemTable},
};

#[panic_handler]
fn _panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "efiapi" fn efi_main(_image_handle: Handle, mut system_table: SystemTable) -> Status {
    let s = cstr16!("Hello, World!\n");
    system_table.stdout().write(s);
    0
}
