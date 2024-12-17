use core::{arch::asm, ffi::c_void, fmt::Display};

use lib::{
    elf::{Elf64Ehdr, Elf64Phdr, ElfClass, ElfDataLayout, ElfMachine, ElfSegmentType, ElfType},
    multiboot2::BootInformationWriter,
    paging::{self, Pml4},
    println,
    uefi::{
        boot_services::BootServices, helper::AllocatedPool, protocols::FileProtocol,
        status::StatusError, AllocateType,
    },
};

#[derive(Debug)]
pub enum KernelHeaderValidationError {
    EfiError(StatusError),
    InvalidMagic,
    InvalidClass,
    InvalidDataLayout,
    InvalidElfType,
    InvalidMachineArch,
}

impl Display for KernelHeaderValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            KernelHeaderValidationError::EfiError(e) => write!(f, "error reading file: {:?}", e),
            KernelHeaderValidationError::InvalidMagic => write!(f, "invalid ELF magic"),
            KernelHeaderValidationError::InvalidClass => {
                write!(f, "invalid ELF class (only 64-bit is supported)")
            }
            KernelHeaderValidationError::InvalidDataLayout => {
                write!(f, "invalid ELF data layout (only LSB is supported)")
            }
            KernelHeaderValidationError::InvalidElfType => {
                write!(f, "invalid ELF type (only ET_EXEC is supported)")
            }
            KernelHeaderValidationError::InvalidMachineArch => {
                write!(
                    f,
                    "invalid ELF machine architecture (only x86_64 is supported)"
                )
            }
        }
    }
}

impl From<StatusError> for KernelHeaderValidationError {
    fn from(value: StatusError) -> Self {
        Self::EfiError(value)
    }
}

struct LoaderMapEntry {
    pub v_addr: u64,
    pub len: u64,
    pub p_addr: u64,
}

impl LoaderMapEntry {
    /// Returns `Some(n)`, where n is the physical address, if the `address` is contained within
    /// this map. Returns `None` if the virtual address is outside the range of this map.
    pub fn translate_virtual_to_physical(&self, address: u64) -> Option<u64> {
        if (self.v_addr..(self.v_addr + self.len)).contains(&address) {
            let offset = address - self.v_addr;
            Some(self.p_addr + offset)
        } else {
            None
        }
    }
}

pub struct ElfKernel {
    elf_header: Elf64Ehdr,
    // For now, store the phdrs to keep them from getting freed (might not be needed)
    _program_headers: AllocatedPool<[Elf64Phdr]>,
    // Maps of virtual addr to physical addr
    map_entries: AllocatedPool<[LoaderMapEntry]>,
}

impl ElfKernel {
    pub fn load_from_file(
        file: &FileProtocol,
        boot_services: BootServices,
    ) -> Result<Self, KernelHeaderValidationError> {
        // Read ELF header
        file.set_position(0)?;
        let mut ehdr: Elf64Ehdr = Default::default();
        if !file.read(&mut ehdr)? {
            return Err(StatusError::LoadError.into());
        };

        Self::validate_header(&ehdr)?;

        // Read program header(s)
        let mut program_headers_pool = AllocatedPool::<[Elf64Phdr]>::try_new(
            boot_services,
            ehdr.program_header_count() as usize,
        )?;
        let program_headers = program_headers_pool.as_mut();
        for phdr in program_headers.iter_mut() {
            if !file.read(phdr)? {
                return Err(StatusError::LoadError.into());
            };
        }

        // Load segments
        // Max 1 map per segment (should be less, since only 1 map per LOAD segment)
        let mut map_entries = AllocatedPool::<[LoaderMapEntry]>::try_new(
            boot_services,
            ehdr.program_header_count() as usize,
        )?;
        for (i, phdr) in program_headers.iter().enumerate() {
            if phdr.p_type() != ElfSegmentType::Load {
                // Segment does not need to be loaded into memory
                continue;
            }
            // Pages are 4KiB each, round up
            let page_count = phdr.p_memsz.div_ceil(0x1000) as usize;
            // Round to the nearest multiple of 4096
            let page_aligned_base = phdr.p_vaddr & !(0x1000 - 1);
            // Difference between the segment's base address (vaddr) and the virtual page's base
            // address.
            let page_offset = phdr.p_vaddr - page_aligned_base;
            let page_base = boot_services.leaky_allocate_pages(
                AllocateType::MaxAddress,
                page_count,
                Some(0x18000),
            )?;

            // Register page map requirement
            map_entries.as_mut()[i] = LoaderMapEntry {
                v_addr: page_aligned_base,
                len: page_count as u64 * 0x1000,
                p_addr: page_base,
            };

            // Load segment into allocated page(s) (with the proper offset into the page)
            file.set_position(phdr.p_offset)?;
            let ptr: *mut c_void = (page_base + page_offset) as *mut c_void;

            // Safety: ptr should be pointing to at least `p_filesz` bytes of available memory
            unsafe { file.read_n_bytes(ptr, phdr.p_filesz as usize) }?;
        }

        Ok(Self {
            elf_header: ehdr,
            _program_headers: program_headers_pool,
            map_entries,
        })
    }

    fn validate_header(ehdr: &Elf64Ehdr) -> Result<(), KernelHeaderValidationError> {
        if !ehdr.valid_magic() {
            return Err(KernelHeaderValidationError::InvalidMagic);
        }

        if ehdr.class() != ElfClass::Class64 {
            return Err(KernelHeaderValidationError::InvalidClass);
        }

        if ehdr.data_layout() != ElfDataLayout::Lsb {
            return Err(KernelHeaderValidationError::InvalidDataLayout);
        }

        if ehdr.elf_type() != ElfType::Executable {
            return Err(KernelHeaderValidationError::InvalidElfType);
        }

        if ehdr.machine() != ElfMachine::X86_64 {
            return Err(KernelHeaderValidationError::InvalidMachineArch);
        }

        Ok(())
    }

    fn entrypoint_addr(&self) -> u64 {
        let v_entry = self.elf_header.e_entry;

        // Translate entrypoint from virtual to physical address
        let mut p_addr = None;
        for map in self.map_entries.as_ref() {
            if let Some(a) = map.translate_virtual_to_physical(v_entry) {
                p_addr = Some(a);
                break;
            }
        }

        p_addr.expect("Could not convert kernel entrypoint to a physical address")
    }

    /// # Safety
    /// This function will panic if the resulting MB2 structure is larger than the allocated
    /// buffer.
    pub fn generate_mb2_info(&self, boot_services: BootServices) -> *mut u8 {
        // Generate the MB2 boot info
        // Lazy: allocate a hard-coded 4096 bytes (will panic if the boot info is larger, unlikely)
        let buf = boot_services
            .leaky_allocate_pages(AllocateType::MaxAddress, 1, Some(0x20000))
            .expect("Error allocating mb2 page");
        // Safety: Writer is bounded by the buffer's allocation
        let writer = unsafe { BootInformationWriter::new(buf as *mut u8, 4096) };
        let mb2_ptr = writer.close();
        println!("D: MB2 info ptr: {:p}", mb2_ptr);
        mb2_ptr
    }

    pub fn call_mb2_entrypoint(&self, mb2_ptr: *mut u8, pml4_ptr: *mut Pml4) -> ! {
        pub const EAX_MB2_MAGIC: u32 = 0x36d76289;
        let entry = self.entrypoint_addr();
        unsafe {
            asm!(
                "mov rbx, {mb2_ptr}",
                // EAX: MB2 magic, RCX: PML4 ptr
                "call rdx",
                in("eax") EAX_MB2_MAGIC,
                mb2_ptr = in(reg) mb2_ptr,
                in("rcx") pml4_ptr,
                in("rdx") entry,
            );
        }
        panic!("Returned from call to kernel!")
    }

    /// Creates the required paging tables for identity mapping the first 4MiB, mapping the kernel
    /// into the upper half and allocating the stack/heap region
    pub fn initialize_paging_structures(&self, boot_services: BootServices) -> &'static mut Pml4 {
        let pml4 = Pml4::new_allocate_empty(boot_services);

        // Identity map the first 4MB
        paging::map_range(boot_services, pml4, 0x0, 0x0, 1024);

        // Upper half mapping
        for loader_map_entry in self.map_entries.as_ref() {
            // Only create new page maps for higher half mappings (This assumes that all the code
            // not found in the higher half is already identity map. This is currently true, as it
            // only includes the bootstrapping assembly code)
            if loader_map_entry.v_addr >= 0xFFFF800000000000 {
                let page_cnt = loader_map_entry.len / 0x1000;
                println!(
                    "D: Mapping {:#x} to {:#x} ({})",
                    loader_map_entry.v_addr, loader_map_entry.p_addr, page_cnt
                );
                paging::map_range(
                    boot_services,
                    pml4,
                    loader_map_entry.v_addr,
                    loader_map_entry.p_addr,
                    page_cnt,
                );
            }
        }

        // Stack & heap (0x80000 to 0x88000)
        let base = boot_services
            .leaky_allocate_pages(AllocateType::Address, 8, Some(0x80000))
            .expect("Error allocating stack memory page");
        paging::map_range(boot_services, pml4, 0x80000, base, 8);

        pml4
    }
}
