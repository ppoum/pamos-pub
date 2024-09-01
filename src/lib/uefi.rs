mod boot_services;
pub mod helper;
pub mod protocols;
pub mod status;
pub mod string;

use core::{ffi::c_void, ptr::NonNull};

use boot_services::{BootServices, RawBootServices};
use protocols::Output;

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
#[derive(Clone, Copy)]
pub struct Handle(NonNull<*mut c_void>);

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
