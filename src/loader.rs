use core::fmt::Display;

use lib::{
    elf::{Elf64Ehdr, ElfClass, ElfDataLayout, ElfMachine, ElfType},
    uefi::{
        protocols::FileProtocol,
        status::{EfiResult, StatusError},
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
                write!(f, "invalid ELF type (only ET_EXEC or ET_DYN is supported)")
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

pub struct KernelFile<'a>(&'a FileProtocol);

impl<'a> KernelFile<'a> {
    pub fn from_ref(reference: &'a FileProtocol) -> Self {
        KernelFile(reference)
    }

    pub fn validate_header(&self) -> Result<(), KernelHeaderValidationError> {
        // Read the ELF header
        self.0.set_position(0)?;
        let header = self.elf_header()?;

        if !header.valid_magic() {
            return Err(KernelHeaderValidationError::InvalidMagic);
        }

        if header.class() != ElfClass::Class64 {
            return Err(KernelHeaderValidationError::InvalidClass);
        }

        if header.data_layout() != ElfDataLayout::Lsb {
            return Err(KernelHeaderValidationError::InvalidDataLayout);
        }

        if header.elf_type() != ElfType::Executable && header.elf_type() != ElfType::Dynamic {
            return Err(KernelHeaderValidationError::InvalidElfType);
        }

        if header.machine() != ElfMachine::X86_64 {
            return Err(KernelHeaderValidationError::InvalidMachineArch);
        }

        Ok(())
    }

    pub fn elf_header(&self) -> EfiResult<Elf64Ehdr> {
        self.0.set_position(0)?;
        let mut header: Elf64Ehdr = Default::default();
        match self.0.read(&mut header)? {
            true => Ok(header),
            false => Err(StatusError::LoadError),
        }
    }
}
