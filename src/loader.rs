use core::{ffi::c_void, fmt::Display};

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

pub struct ElfKernel {
    elf_header: Elf64Ehdr,
    // For now, store the phdrs to keep them from getting freed (might not be needed)
    _program_headers: AllocatedPool<[Elf64Phdr]>,
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
        for phdr in program_headers {
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

    // # Safety
    // The ELF entrypoint must not expect any arguments, and should return a usize
    pub fn entrypoint(&self) -> unsafe extern "C" fn() -> usize {
        let ptr = self.elf_header.e_entry as *const ();
        unsafe { core::mem::transmute(ptr) }
    }
}
