#![no_std]
#![no_main]

mod loader;

use lib::{
    cstr16, println,
    uefi::{
        helper::{self, AllocatedPool},
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
        Ok(p) => p,
        Err(ProtocolLocateError::Unsupported) => panic!("Unsupported protocol"),
        Err(ProtocolLocateError::Error(_)) => panic!("Other error"),
    }
}

#[no_mangle]
pub extern "efiapi" fn efi_main(image_handle: Handle, mut system_table: SystemTable) -> Status {
    helper::register_services(&system_table);
    let boot_services = system_table.boot_services();

    let res = LoadedImageProtocol::try_locate_from_handle(image_handle, &boot_services);
    let loaded_image = unwrap_protocol_result(res);

    // Get volume from our EFI app handle and open root path
    let res =
        SimpleFileSystemProtocol::try_locate_from_handle(loaded_image.device(), &boot_services);
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

    let kernel = ElfKernel::load_from_file(kernel_file, system_table.boot_services())
        .expect("error reading kernel file");

    println!("I: Kernel file loaded");

    println!("I: Generating the MB2 info structure");
    // Lazy: allocate a hard-coded 1000 bytes (will panic if the boot info is larger)
    let mut buf =
        AllocatedPool::<[u8]>::try_new(boot_services, 1000).expect("Error allocating MB2 buffer");
    let mb2_ptr = kernel.generate_mb2_info(&mut buf);
    let pml4_ptr = kernel.initialize_paging_structures(boot_services);

    println!("I: Exiting boot services and entering kernel");
    let mmap = boot_services
        .memory_map()
        .expect("Error getting memory map");

    // Avoid printing, it seems like SimpleTextOutputProtocol.OutputString sometimes allocates?
    boot_services
        .exit_boot_services(image_handle, mmap.key())
        .expect("Error exiting boot services");

    kernel.call_mb2_entrypoint(mb2_ptr, pml4_ptr as *mut _);
}
