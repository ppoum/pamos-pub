use definitions::{
    Elf64Addr, Elf64Half, Elf64Off, Elf64Word, Elf64XWord, EI_CLASS, EI_DATA, EI_MAG0, EI_MAG3,
    EI_NIDENT, ELFMAG,
};

mod definitions;

pub use definitions::types::*;

#[repr(C)]
#[derive(Default, Debug)]
pub struct Elf64Ehdr {
    e_ident: [u8; EI_NIDENT],
    e_type: Elf64Half,
    e_machine: Elf64Half,
    e_version: Elf64Half,
    /// Entry point virtual address
    pub e_entry: Elf64Addr,
    /// Program header table file offset
    e_phoff: Elf64Off,
    /// Section header table file offset
    e_shoff: Elf64Off,
    e_flags: Elf64Word,
    e_ehsize: Elf64Half,
    e_phentsize: Elf64Half,
    e_phnum: Elf64Half,
    e_shentsize: Elf64Half,
    e_shnum: Elf64Half,
    e_shstrndx: Elf64Half,
}

impl Elf64Ehdr {
    pub fn valid_magic(&self) -> bool {
        &self.e_ident[EI_MAG0..EI_MAG3 + 1] == ELFMAG.as_bytes()
    }

    pub fn class(&self) -> ElfClass {
        self.e_ident[EI_CLASS].into()
    }

    pub fn data_layout(&self) -> ElfDataLayout {
        self.e_ident[EI_DATA].into()
    }

    pub fn elf_type(&self) -> ElfType {
        self.e_type.into()
    }

    pub fn version(&self) -> ElfVersion {
        self.e_version.into()
    }

    pub fn machine(&self) -> ElfMachine {
        self.e_machine.into()
    }

    pub fn program_header_count(&self) -> Elf64Half {
        self.e_phnum
    }
}

#[repr(C)]
#[derive(Default, Debug)]
pub struct Elf64Phdr {
    p_type: Elf64Word,
    pub p_flags: Elf64Word,
    /// Segment file offset
    pub p_offset: Elf64Off,
    /// Segment virtual address
    pub p_vaddr: Elf64Addr,
    /// Segment physical address
    pub p_paddr: Elf64Addr,
    /// Segment size in file
    pub p_filesz: Elf64XWord,
    /// Segment size in memory
    pub p_memsz: Elf64XWord,
    /// Segment alignment, file & memory
    pub p_align: Elf64XWord,
}

impl Elf64Phdr {
    pub fn p_type(&self) -> ElfSegmentType {
        self.p_type.into()
    }
}
