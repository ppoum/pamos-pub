#![no_std]
#![no_main]

mod loader;

use lib::{
    cstr16, println,
    uefi::{
        boot_services::BootServices,
        helper::{self},
        protocols::{
            FileAttribute, FileMode, GraphicsOutputProtocol, LoadedImageProtocol, Protocol,
            ProtocolLocateError, SimpleFileSystemProtocol,
        },
        status::{EfiResult, Status, StatusError},
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

    configure_framebuffer(&boot_services).unwrap();

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
    let mb2_ptr = kernel.generate_mb2_info(boot_services);
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

/// Initialize the Graphics Output device (if required), and set the mode to the highest resolution
/// available (where highest means largest pixel count).
pub fn configure_framebuffer(boot_services: &BootServices) -> EfiResult<()> {
    let graphics_protocol =
        unwrap_protocol_result(GraphicsOutputProtocol::try_locate(boot_services));
    // If mode is null, use mode number 0
    let current_mode = graphics_protocol.mode().map(|m| m.mode).unwrap_or_default();
    match graphics_protocol.query_mode(current_mode) {
        Ok(_) => {}
        Err(StatusError::NotStarted) => {
            // Not started, set mode 0
            graphics_protocol.set_mode(0)?;
        }
        Err(e) => panic!("Unexpected error configuring framebuffer: {:?}", e),
    };

    let (min, max) = graphics_protocol
        .mode()
        .map(|m| (m.mode, m.max_mode))
        .expect("FB mode should not be null");

    // Select the mode with the highest resolution
    // NOTE: Is there a better heuristic we can use here?
    let mut highest = 0;
    let mut highest_idx = 0;
    let mut highest_info = None;
    for i in min..max {
        let mode = graphics_protocol.query_mode(i)?;
        let res_cnt = mode.vertical_res * mode.horizontal_res;
        if res_cnt > highest {
            highest = res_cnt;
            highest_idx = i;
            highest_info = Some(mode);
        }
    }

    // If this is still None, then unexpected modes in loop above
    let highest_info = highest_info.unwrap();
    if current_mode != highest_idx {
        graphics_protocol.set_mode(highest_idx)?;
        println!(
            "D: Changing FB mode to {}x{}",
            highest_info.horizontal_res, highest_info.vertical_res
        );
    }

    Ok(())
}
