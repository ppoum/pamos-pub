use core::ffi::c_void;

use uefi_macros::Protocol;

use crate::{guid, uefi::Guid};

use super::RawProtocol;

#[repr(transparent)]
#[derive(Protocol)]
pub struct SimpleFileSystemProtocol(RawSimpleFileSystemProtocol);

impl SimpleFileSystemProtocol {}

#[repr(C)]
struct RawSimpleFileSystemProtocol {
    revision: u64,
    open_volume: unsafe extern "efiapi" fn(this: *mut Self, root: *mut *const RawFileProtocol),
}

impl RawProtocol for RawSimpleFileSystemProtocol {
    const GUID: Guid = guid!("964E5B22-6459-11D2-8E39-00A0C969723B");
}

#[repr(transparent)]
pub struct FileProtocol(RawFileProtocol);

impl FileProtocol {}

#[repr(C)]
struct RawFileProtocol {
    revision: u64,
    open: *const c_void,
    close: *const c_void,
    delete: *const c_void,
    read: *const c_void,
    write: *const c_void,
    get_position: *const c_void,
    set_position: *const c_void,
    flush: *const c_void,
    open_ex: *const c_void,
    read_ex: *const c_void,
    write_ex: *const c_void,
    flush_ex: *const c_void,
}
