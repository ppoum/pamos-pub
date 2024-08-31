use core::ffi::c_void;

use crate::guid;

use super::{
    boot_services::BootServices,
    status::{Status, StatusError},
    Guid, Handle, MemoryType, RawSystemTable,
};

pub enum ProtocolLocateError {
    Unsupported,
    Error(StatusError),
}

pub trait RawProtocol: Sized {
    const GUID: Guid;

    fn try_locate_protocol(
        boot_services: &BootServices,
        handle: Handle,
    ) -> Result<*const Self, ProtocolLocateError> {
        let res = boot_services.generic_handle_protocol(handle, &Self::GUID);
        let void_interface = match res {
            Ok(Some(x)) => x,
            Ok(None) => return Err(ProtocolLocateError::Unsupported),
            Err(e) => return Err(ProtocolLocateError::Error(e)),
        };

        Ok(void_interface as *const Self)
    }
}

#[repr(C)]
pub(crate) struct RawSimpleTextOutputProtocol {
    pub reset: unsafe extern "efiapi" fn(this: *mut Self, extended_verification: bool) -> Status,
    pub output_string: unsafe extern "efiapi" fn(this: *mut Self, string: *const u16) -> Status,
    pub test_string: unsafe extern "efiapi" fn(this: *mut Self, string: *const u16) -> Status,
    pub query_mode: unsafe extern "efiapi" fn(
        this: *mut Self,
        mode_number: usize,
        colums: *mut usize,
        rows: *mut usize,
    ) -> Status,
    pub set_mode: unsafe extern "efiapi" fn(this: *mut Self, mode_num: usize) -> Status,
    pub set_attribute: unsafe extern "efiapi" fn(this: *mut Self, attr: usize) -> Status,
    pub clear_screen: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
    pub set_cursor_position:
        unsafe extern "efiapi" fn(this: *mut Self, col: usize, row: usize) -> Status,
    pub enable_cursor: unsafe extern "efiapi" fn(this: *mut Self, visible: bool) -> Status,
    mode: usize, // TODO
}

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
