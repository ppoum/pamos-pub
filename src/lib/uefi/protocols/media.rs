use core::{ffi::c_void, ptr};

use uefi_macros::Protocol;

use crate::{
    guid,
    uefi::{
        status::{EfiResult, Status},
        Guid,
    },
};

use super::RawProtocol;

#[repr(transparent)]
#[derive(Protocol)]
pub struct SimpleFileSystemProtocol(RawSimpleFileSystemProtocol);

impl SimpleFileSystemProtocol {
    pub fn open_volume(&self) -> EfiResult<&FileProtocol> {
        let self_ptr = (&self.0 as *const RawSimpleFileSystemProtocol).cast_mut();
        let mut root: *const RawFileProtocol = ptr::null();
        let root_ptr: *mut *const RawFileProtocol = &mut root;

        // Safety: Only assumes self is a valid FS Protocol
        unsafe { (self.0.open_volume)(self_ptr, root_ptr) }.to_result()?;

        // Safety: Assumes root is a valid pointer (checking the status above)
        unsafe { Ok(&*(root as *const FileProtocol)) }
    }
}

#[repr(C)]
struct RawSimpleFileSystemProtocol {
    revision: u64,
    open_volume:
        unsafe extern "efiapi" fn(this: *mut Self, root: *mut *const RawFileProtocol) -> Status,
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
