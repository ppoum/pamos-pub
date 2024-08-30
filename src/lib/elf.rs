use definitions::{
    Elf64Addr, Elf64Half, Elf64Off, Elf64Word, Elf64XWord, EI_CLASS, EI_DATA, EI_MAG0, EI_MAG3,
    EI_NIDENT, ELFMAG,
};

mod definitions;

pub use definitions::types::*;

#[repr(C)]
pub struct Elf64Ehdr {
    pub e_ident: [u8; EI_NIDENT],
    pub e_type: Elf64Half,
    pub e_machine: Elf64Half,
    pub e_version: Elf64Half,
    /// Entry point virtual address
    pub e_entry: Elf64Addr,
    /// Program header table file offset
    pub e_phoff: Elf64Off,
    /// Section header table file offset
    pub e_shoff: Elf64Off,
    pub e_flags: Elf64Word,
    pub e_ehsize: Elf64Half,
    pub e_phentsize: Elf64Half,
    pub e_phnum: Elf64Half,
    pub e_shentsize: Elf64Half,
    pub e_shnum: Elf64Half,
    pub e_shstrndx: Elf64Half,
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
}

#[repr(C)]
pub struct Elf64Phdr {
    pub p_type: Elf64Word,
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
