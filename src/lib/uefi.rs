mod boot_services;
pub mod io;
pub mod protocol;
pub mod status;

use core::{ffi::c_void, ptr::NonNull, slice};

use boot_services::{BootServices, RawBootServices};
use io::Output;

#[repr(transparent)]
pub struct CStr16([u16]);

impl CStr16 {
    /// Converts a u16 slice to a &CStr16.
    /// # Safety
    /// This function assumes the slice is null-terminated and that
    /// every character is a valid UCS-2 character. Failure to match the
    /// above conditions may result in undefined behaviour.
    pub const unsafe fn from_u16_unsafe(chars: &[u16]) -> &Self {
        &*(chars as *const [u16] as *const Self)
    }

    /// Converts a pointer to a u16 array to a &CStr16.
    /// # Safety
    /// The pointer should point to a valid memory location that contains a valid
    /// UCS-2 string. Failure to due so may result in the string being arbitrarily long
    /// and/or have invalid characters.
    pub unsafe fn from_ptr<'a>(ptr: *const u16) -> &'a Self {
        let mut length = 0;
        while *ptr.add(length) != 0 {
            length += 1;
        }

        Self::from_u16_unsafe(slice::from_raw_parts(ptr, length))
    }

    pub const fn as_ptr(&self) -> *const u16 {
        self.0.as_ptr()
    }
}

#[repr(C)]
pub enum MemoryType {
    EfiReservedMemoryType,
    EfiLoaderCode,
    EfiLoaderData,
    EfiBootServicesCode,
    EfiBootServicesData,
    EfiRuntimeServicesCode,
    EfiRuntimeServicesData,
    EfiConventionalMemory,
    EfiUnusableMemory,
    EfiACPIReclaimMemory,
    EfiACPIMemoryNVS,
    EfiMemoryMappedIO,
    EfiMemoryMappedIOPortSpace,
    EfiPalCode,
    EfiPersistentMemory,
    EfiUnaceptedMemoryType,
    EfiMaxMemoryType,
}

#[repr(transparent)]
pub struct Guid([u8; 16]);

impl Guid {
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
}

#[repr(transparent)]
pub struct Handle(NonNull<c_void>);

#[repr(transparent)]
pub struct SystemTable(*const RawSystemTable);

impl SystemTable {
    pub fn stdout(&mut self) -> &mut Output {
        unsafe { &mut *(*self.0).con_out }
    }

    pub fn boot_services(&mut self) -> BootServices {
        unsafe {
            let x = &*self.0;
            let x = x.boot_services;
            BootServices::from_ptr(x)
        }
    }
}

#[repr(C)]
pub struct TableHeader {
    pub signature: u64,
    pub revision: u32,
    pub header_size: u32,
    pub crc32: u32,
    _reserved: u32,
}

#[repr(C)]
pub struct RawSystemTable {
    hdr: TableHeader,
    firmware_vendor: *const u16,
    firmware_revision: u32,
    console_in_handle: Handle,
    con_in: *const c_void, // TODO
    console_out_handle: Handle,
    con_out: *mut Output,
    std_err_handle: Handle,
    std_err: *const c_void,
    runtime_services: *const c_void, // TODO
    boot_services: *mut RawBootServices,
    num_table_entries: usize,
    config_table: *const c_void, // TODO
}
