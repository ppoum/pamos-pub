#![no_std]
#![no_main]

use uefi::{Handle, Status, SystemTable};
use uefi_macros::cstr16;

#[panic_handler]
fn _panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "efiapi" fn efi_main(_image_handle: Handle, system_table: &SystemTable) -> Status {
    let s = cstr16!("Hello, World!\n");
    let x = unsafe { &mut *system_table.con_out };
    x.write(s);
    0
}
