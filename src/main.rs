#![no_std]
#![no_main]

mod loader;

use lib::{
    cstr16,
    multiboot2::BootInformationWriter,
    println,
    uefi::{
        boot_services::BootServices,
        helper::{self},
        protocols::{
            FileAttribute, FileMode, GraphicsOutputProtocol, LoadedImageProtocol, Protocol,
            ProtocolLocateError, SimpleFileSystemProtocol,
        },
        status::{EfiResult, Status, StatusError},
        AllocateType, Handle, SystemTable,
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

    println!(
        "I: Kernel file loaded (entry: {:#x})",
        kernel.entrypoint_addr()
    );

    println!("I: Generating the MB2 info structure");
    let mb2_ptr =
        generate_mb2_info_structure(&boot_services).expect("Error writing MB2 info structure");
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
fn configure_framebuffer(boot_services: &BootServices) -> EfiResult<()> {
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

fn generate_mb2_info_structure(boot_services: &BootServices) -> EfiResult<*mut u8> {
    // Lazy: allocate a hard-coded 4096 bytes (will panic if the boot info is larger, unlikely)
    let buf = boot_services
        .leaky_allocate_pages(AllocateType::MaxAddress, 1, Some(0x80000))
        .expect("Error allocating mb2 page");
    // Safety: Writer is bounded by the buffer's allocation
    let mut writer = unsafe { BootInformationWriter::new(buf as *mut u8, 4096) };

    // Framebuffer tag
    {
        let gop =
            GraphicsOutputProtocol::try_locate(boot_services).map_err(|_| StatusError::NotFound)?;
        let gop_info = gop.mode().ok_or(StatusError::NotFound)?;
        let current_mode = gop.query_mode(gop_info.mode)?;
        writer.write_framebuffer_tag(gop_info, current_mode);
    }

    let mb2_ptr = writer.close();
    println!("D: MB2 info ptr: {:p}", mb2_ptr);
    Ok(mb2_ptr)
}
