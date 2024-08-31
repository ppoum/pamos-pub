use core::ffi::c_void;

use crate::{
    guid,
    uefi::{boot_services::BootServices, status::Status, Guid, Handle, MemoryType, RawSystemTable},
};

use super::{ProtocolLocateError, RawProtocol};

#[repr(transparent)]
pub struct LoadedImageProtocol(*const RawEfiLoadedImageProtocol);

impl LoadedImageProtocol {
    pub fn try_locate(
        handle: Handle,
        boot_services: &BootServices,
    ) -> Result<Self, ProtocolLocateError> {
        let raw = RawEfiLoadedImageProtocol::try_locate_protocol(boot_services, handle)?;
        Ok(Self(raw))
    }
}

#[repr(C)]
pub struct RawEfiLoadedImageProtocol {
    pub revision: u32,
    pub parent_handle: Handle,
    pub system_table: *const RawSystemTable,
    pub device_handle: Handle,
    file_path: usize, // TODO: EFI_DEVICE_PATH_PROTOCOL
    _reserved: *const c_void,
    pub load_options_size: u32,
    pub load_options: *const c_void,
    pub image_base: *const c_void,
    pub image_size: u64,
    pub image_code_type: MemoryType,
    pub image_data_type: MemoryType,
    pub unload: unsafe extern "efiapi" fn(image_handle: Handle) -> Status,
}

impl RawProtocol for RawEfiLoadedImageProtocol {
    const GUID: Guid = guid!("5B1B31A1-9562-11D2-8E3F-00A0C969723B");
}
