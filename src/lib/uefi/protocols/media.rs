use core::{ffi::c_void, ptr};

use bitflags::bitflags;
use uefi_macros::Protocol;

use crate::{
    guid,
    uefi::{
        status::{EfiResult, Status, StatusError},
        string::CStr16,
        Guid,
    },
};

use super::RawProtocol;

bitflags! {
    #[repr(transparent)]
    #[derive(PartialEq, Eq)]
    pub struct FileMode: u64 {
        const Read      = 0x0000000000000001;
        const Write     = 0x0000000000000002;
        const Create    = 0x8000000000000000;
        const _         = !0;
    }

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Default)]
    pub struct FileAttribute : u64 {
        const ReadOnly  = 0x0000000000000001;
        const Hidden    = 0x0000000000000002;
        const System    = 0x0000000000000004;
        const Reserved  = 0x0000000000000008;
        const Directory = 0x0000000000000010;
        const Archive   = 0x0000000000000020;
        const ValidAttr = 0x0000000000000037;
        const _         = !0;
    }
}

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

impl FileProtocol {
    pub fn open(
        &self,
        file_name: &CStr16,
        open_mode: FileMode,
        attributes: FileAttribute,
    ) -> EfiResult<&FileProtocol> {
        // Validate file mode
        if !(open_mode == FileMode::Read
            || open_mode == FileMode::Read | FileMode::Write
            || open_mode == FileMode::Read | FileMode::Write | FileMode::Create)
        {
            return Err(StatusError::InvalidParameter);
        }

        if !open_mode.intersects(FileMode::Create) && !attributes.is_empty() {
            // Attributes can only be used when creating a file
            return Err(StatusError::InvalidParameter);
        }

        let mut new_handle: *const RawFileProtocol = ptr::null();
        let new_handle_ptr: *mut *const RawFileProtocol = &mut new_handle;

        // Safety: File name must be a valid string, file mode can only have a few combinations,
        // attributes only when creating a file
        unsafe {
            (self.0.open)(
                &self.0 as *const _ as *mut _,
                new_handle_ptr,
                file_name.as_ptr(),
                open_mode.bits(),
                attributes.bits(),
            )
        }
        .to_result()?;

        // Safety: If call succeeds (checked above), then should be a valid pointer
        unsafe { Ok(&*(new_handle as *const _)) }
    }

    /// Reads a file into an object. Returns `Ok(true)` if it filled the buffer, `Ok(false)` if the
    /// buffer wasn't filled, and `Err` if there was an error.
    pub fn read<T: Sized>(&self, buf: &mut T) -> EfiResult<bool> {
        let t_size = size_of::<T>();
        let mut buf_size = t_size;

        // Safety: Assuming the self and buf references are valid, no safety restrictions can be
        // violated during this function call.
        unsafe {
            (self.0.read)(
                self as *const _ as *mut _,
                &mut buf_size as *mut _,
                buf as *mut T as _,
            )
        }
        .to_result()?;

        Ok(t_size == buf_size)
    }

    /// Reads a file into memory, starting at `ptr` until `ptr+n`. Returns `Ok(true)` if n bytes
    /// were read, `Ok(false)` if less than n bytes were read, and `Err` if an error occurred.
    ///
    /// # Safety
    /// The pointer must point to a valid memory location, with the following `n` bytes usable.
    pub unsafe fn read_n_bytes(&self, ptr: *mut c_void, n: usize) -> EfiResult<bool> {
        let mut buf_size = n;
        unsafe { (self.0.read)(self as *const _ as *mut _, &mut buf_size as *mut _, ptr) }
            .to_result()?;

        Ok(buf_size == n)
    }

    pub fn set_position(&self, position: u64) -> EfiResult<()> {
        // Safety: Assumes self is a valid reference
        unsafe { (self.0.set_position)(self as *const _ as *mut _, position) }.to_result()
    }
}

#[repr(C)]
struct RawFileProtocol {
    revision: u64,
    open: unsafe extern "efiapi" fn(
        this: *mut Self,
        new_handle: *mut *const RawFileProtocol,
        file_name: *const u16,
        open_mode: u64,
        attributes: u64,
    ) -> Status,
    close: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
    delete: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
    read: unsafe extern "efiapi" fn(
        this: *mut Self,
        buffer_size: *mut usize,
        buffer: *mut c_void,
    ) -> Status,
    write: *const c_void,
    get_position: *const c_void,
    set_position: unsafe extern "efiapi" fn(this: *mut Self, position: u64) -> Status,
    flush: *const c_void,
    open_ex: *const c_void,
    read_ex: *const c_void,
    write_ex: *const c_void,
    flush_ex: *const c_void,
}
