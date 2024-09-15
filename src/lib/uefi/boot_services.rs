use core::{ffi::c_void, ptr};

use crate::{
    print,
    uefi::{helper::AllocatedPool, status::StatusError},
};

use super::{
    status::{EfiResult, Status},
    AllocateType, Guid, Handle, MemoryType, PhysicalAddress, TableHeader, VirtualAddress,
};

#[repr(transparent)]
#[derive(Clone, Copy)]
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

    /// Returns `Ok(ptr)` if the call succeeded, where `ptr` points to the start of the pool.
    /// Returns `Err` otherwise.
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

    pub(crate) fn free_pool<T: ?Sized>(&self, buf: *mut T) -> EfiResult<()> {
        // Safety: If the buffer reference doesn't point to an allocated pool, the function returns
        // an error status, so no unwanted action can be taken.
        unsafe { ((*self.0).free_pool)(buf as *mut c_void) }.to_result()
    }

    /// Allocates `n` 4KiB pages for use. `address` can only be none if using [`AllocateType::AnyPages`].
    /// # Safety
    /// The caller must ensure that the provided address is valid. This function allocate the
    /// page(s) and returns a raw pointer to the pages. It does not try to free the allocation. *The
    /// caller* needs to free the pages if they are no longer in use.
    pub fn leaky_allocate_pages(
        &self,
        allocate_type: AllocateType,
        pages: usize,
        address: Option<PhysicalAddress>,
    ) -> EfiResult<PhysicalAddress> {
        if allocate_type != AllocateType::AnyPages && address.is_none() {
            // Missing address
            return Err(StatusError::InvalidParameter);
        }
        let mut address = address.unwrap_or_default();

        // Safety: If requirements from function doc are met, no safety issues.
        unsafe {
            ((*self.0).allocate_pages)(
                allocate_type,
                MemoryType::EfiLoaderCode,
                pages,
                &mut address as *mut _,
            )
        }
        .to_result()?;

        Ok(address)
    }

    pub fn free_pages(&self, memory: PhysicalAddress, pages: usize) -> EfiResult<()> {
        unsafe { ((*self.0).free_pages)(memory, pages) }.to_result()
    }

    pub fn memory_map(&self) -> EfiResult<MemoryMap> {
        // Since we allocate between this call and the next call to GetMemoryMap, double the buffer
        // size needed to ensure the 2 allocations below don't cause the map to get bigger than the
        // buffer.
        let mut map_size = 2 * self.memory_map_size()?;

        let mut pool = AllocatedPool::<[u8]>::try_new(*self, map_size)?;
        let buf = pool.as_mut();

        // NOTE: Avoid allocating after the call to GetMemoryMap (estimate the size)
        let mut desc_pool = {
            let desc_count_overestimate = map_size / size_of::<MemoryDescriptor>();
            AllocatedPool::<[MemoryDescriptor]>::try_new(*self, desc_count_overestimate)?
        };

        let mut map_key = 0_usize;
        let mut desc_size = 0_usize;
        let mut desc_version = 0_usize;

        unsafe {
            ((*self.0).get_memory_map)(
                &mut map_size as *mut _,
                buf.as_mut_ptr() as *mut _,
                &mut map_key as *mut _,
                &mut desc_size as *mut _,
                &mut desc_version as *mut _,
            )
        }
        .to_result()?;

        let descriptor_count = map_size / desc_size;

        // NOTE: We could probably make this cleaner and avoid "wasting" one allocation by simply
        // including this logic within the MemoryMap implementation, thus removing the need to copy
        // the map into the new struct.
        for i in 0..descriptor_count {
            let desc = unsafe { &*(buf.as_ptr().add(i * desc_size) as *const MemoryDescriptor) };
            desc_pool[i] = *desc;
        }

        Ok(MemoryMap::new(desc_pool, descriptor_count, map_key))
    }

    /// Gets the buffer size needed to hold the memory map by calling GetMemoryMap with a buffer
    /// size of 0.
    fn memory_map_size(&self) -> EfiResult<usize> {
        // Call GetMemoryMap with a buffer size of 0, expect BufferTooSmall
        let mut map_size = 0_usize;
        let mut map_key = 0_usize;
        let mut descriptor_size = 0_usize;
        let mut descriptor_version = 0_usize;

        let res = unsafe {
            ((*self.0).get_memory_map)(
                &mut map_size as *mut _,
                ptr::null_mut() as *mut _,
                &mut map_key as *mut _,
                &mut descriptor_size as *mut _,
                &mut descriptor_version as *mut _,
            )
        }
        .to_result();

        match res {
            Ok(()) => panic!("Call to GetMemoryMap succeeded when it should have failed!"),
            Err(StatusError::BufferTooSmall) => {}
            Err(e) => return Err(e),
        }

        Ok(map_size)
    }
}

#[repr(C)]
pub(crate) struct RawBootServices {
    hdr: TableHeader,
    // Task Priority Services
    raise_tpl: *const c_void,
    restore_tpl: *const c_void,

    // Memory Services
    allocate_pages: unsafe extern "efiapi" fn(
        allocate_type: AllocateType,
        memory_type: MemoryType,
        pages: usize,
        address: *mut PhysicalAddress,
    ) -> Status,
    free_pages: unsafe extern "efiapi" fn(memory: PhysicalAddress, pages: usize) -> Status,
    get_memory_map: unsafe extern "efiapi" fn(
        map_size: *mut usize,
        memory_map: *mut MemoryDescriptor,
        map_key: *mut usize,
        descriptor_size: *mut usize,
        descriptor_version: *mut usize,
    ) -> Status,
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryDescriptor {
    descriptor_type: MemoryType,
    physical_address: PhysicalAddress,
    virtual_address: VirtualAddress,
    num_pages: u64,
    attribute: u64,
}

pub struct MemoryMap {
    descriptors: AllocatedPool<[MemoryDescriptor]>,
    descriptor_count: usize,
    map_key: usize,
}

impl MemoryMap {
    pub fn new(
        descriptors: AllocatedPool<[MemoryDescriptor]>,
        descriptor_count: usize,
        map_key: usize,
    ) -> Self {
        Self {
            descriptors,
            descriptor_count,
            map_key,
        }
    }

    pub fn key(&self) -> usize {
        self.map_key
    }
}
