use core::{arch::asm, ffi::c_void, fmt::Display, ops::RangeBounds};

use lib::{
    elf::{Elf64Ehdr, Elf64Phdr, ElfClass, ElfDataLayout, ElfMachine, ElfSegmentType, ElfType},
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
            let page_base =
                boot_services.leaky_allocate_pages(AllocateType::AnyPages, page_count, None)?;

            // Register page map requirement
            map_entries.as_mut()[i] = LoaderMapEntry {
                v_addr: page_aligned_base,
                len: page_count as u64 * 0x1000,
                p_addr: page_base,
            };

            // Load segment into allocated page(s) (with the proper offset into the page)
            file.set_position(phdr.p_offset)?;
            let ptr: *mut c_void = (page_base + page_offset) as *mut c_void;
            println!(
                "DEBUG: Need to create page mapping physical {:#x} to {:#x}",
                page_base, page_aligned_base
            );

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
    /// The ELF entrypoint must not expect any arguments, and should return a usize
    pub fn entrypoint(&self) -> unsafe extern "C" fn() -> usize {
        let ptr = self.entrypoint_addr() as *const ();
        unsafe { core::mem::transmute(ptr) }
    }
}
