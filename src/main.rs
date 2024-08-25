#![no_std]
#![no_main]

use pamos_pub::{Handle, Status, SystemTable};

#[panic_handler]
fn _panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "efiapi" fn efi_main(_image_handle: Handle, _system_table: &SystemTable) -> Status {
    0
}
