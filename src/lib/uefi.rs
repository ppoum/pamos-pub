pub mod io;
mod protocol;

use core::{ffi::c_void, ptr::NonNull};

use io::Output;

pub type Status = usize;

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

    pub const fn as_ptr(&self) -> *const u16 {
        self.0.as_ptr()
    }
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
}

#[repr(C)]
struct RawSystemTable {
    hdr: TableHeader,
    firmware_vendor: usize, // TODO CStr
    firmware_revision: u32,
    console_in_handle: Handle,
    con_in: usize, // TODO
    console_out_handle: Handle,
    con_out: *mut Output,
    std_err_handle: Handle,
    std_err: usize,
    runtime_services: usize, // TODO
    boot_services: usize,    // TODO
    num_table_entries: usize,
    config_table: usize, // TODO
}

#[repr(C)]
pub struct TableHeader {
    signature: u64,
    reivsion: u32,
    header_size: u32,
    crc32: u32,
    _reserved: u32,
}
