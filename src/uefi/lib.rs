#![no_std]

use core::{ffi::c_void, ptr::NonNull};

pub type Status = usize;

#[repr(transparent)]
pub struct Handle(NonNull<c_void>);

pub struct SystemTable {
    hdr: usize,             // TODO
    firmware_vendor: usize, // TODO CStr
    firmware_revision: u32,
    console_in_handle: Handle,
    con_in: usize, // TODO
    console_out_handle: Handle,
    con_out: usize, // TODO
    std_err_handle: Handle,
    std_err: usize,
    runtime_services: usize, // TODO
    boot_services: usize,    // TODO
    num_table_entries: usize,
    config_table: usize, // TODO
}
