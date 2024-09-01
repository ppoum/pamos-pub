use core::{ffi::c_void, ptr};

use crate::uefi::status::StatusError;

use super::{
    status::{EfiResult, Status},
    Guid, Handle, MemoryType, TableHeader,
};

#[repr(transparent)]
pub struct BootServices(*mut RawBootServices);

impl BootServices {
    pub(crate) fn from_ptr(ptr: *mut RawBootServices) -> Self {
        Self(ptr)
    }

    pub(crate) fn generic_handle_protocol(
        &self,
        handle: Handle,
        protocol: &Guid,
    ) -> EfiResult<Option<*const c_void>> {
        let mut interface: *const c_void = ptr::null();
        let interface_ptr: *mut *const c_void = &mut interface;
        // // Safety: Handled on the EFI side, our data structures aren't null
        let result =
            unsafe { ((*self.0).handle_protocol)(handle, protocol, interface_ptr) }.to_result();

        match result {
            Ok(()) => Ok(Some(interface)),
            Err(StatusError::Unsupported) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub(crate) fn allocate_pool(&self, size: usize) -> EfiResult<*mut c_void> {
        self.allocate_pool_with_mem_type(MemoryType::EfiLoaderData, size)
    }

    pub(crate) fn allocate_pool_with_mem_type(
        &self,
        mem_type: MemoryType,
        size: usize,
    ) -> EfiResult<*mut c_void> {
        let mut buf: *mut c_void = ptr::null_mut();
        let buf_ptr: *mut *mut c_void = &mut buf;

        unsafe { ((*self.0).allocate_pool)(mem_type, size, buf_ptr) }.to_result()?;

        Ok(buf)
    }

    pub(crate) fn free_pool<T>(&self, buf: *mut T) -> EfiResult<()> {
        // Safety: If the buffer reference doesn't point to an allocated pool, the function returns
        // an error status, so no unwanted action can be taken.
        unsafe { ((*self.0).free_pool)(buf as *mut c_void) }.to_result()
    }
}

#[repr(C)]
pub(crate) struct RawBootServices {
    hdr: TableHeader,
    // Task Priority Services
    raise_tpl: *const c_void,
    restore_tpl: *const c_void,

    // Memory Services
    allocate_pages: *const c_void,
    free_pages: *const c_void,
    get_memory_map: *const c_void,
    allocate_pool: unsafe extern "efiapi" fn(
        pool_type: MemoryType,
        size: usize,
        buffer: *mut *mut c_void,
    ) -> Status,
    free_pool: unsafe extern "efiapi" fn(buffer: *mut c_void) -> Status,

    // Event & Timer Services
    create_event: *const c_void,
    set_timer: *const c_void,
    wait_for_event: *const c_void,
    signal_event: *const c_void,
    close_event: *const c_void,
    check_event: *const c_void,

    // Protocol Handler Services
    install_protocol_interface: *const c_void,
    reinstall_protocol_interface: *const c_void,
    uninstall_protocol_interface: *const c_void,
    handle_protocol: unsafe extern "efiapi" fn(
        handle: Handle,
        protocol: *const Guid,
        interface: *mut *const c_void,
    ) -> Status,
    _reserved: *const c_void,
    register_protocol_notify: *const c_void,
    locate_handle: *const c_void,
    locate_device_path: *const c_void,
    install_configuration_table: *const c_void,

    // Image Services
    load_image: *const c_void,
    start_image: *const c_void,
    exit: *const c_void,
    unload_image: *const c_void,
    exit_boot_services: *const c_void,

    // Miscellaneous Services
    get_next_monotonic_count: *const c_void,
    stall: *const c_void,
    set_watchdog_timer: *const c_void,

    // DriverSupport Services
    connect_controller: *const c_void,
    disconnect_controller: *const c_void,

    // Open and Close Protocol Services
    open_protocol: *const c_void,
    close_protocol: *const c_void,
    open_protocol_information: *const c_void,

    // Library Services
    protocols_per_handle: *const c_void,
    locate_handle_buffer: *const c_void,
    locate_protocol: *const c_void,
    install_multiple_protocol_interfaces: *const c_void,
    uninstall_multiple_protocol_interfaces: *const c_void,

    // 32-bit CRC Services
    calculate_crc32: *const c_void,

    // Miscellaneous Services
    copy_mem: *const c_void,
    set_mem: *const c_void,
    create_event_ex: *const c_void,
}
