// Definitions taken from elf.h on Linux 6-10.6-gentoo
#![allow(unused)]

pub type Elf64Addr = u64;
pub type Elf64Half = u16;
pub type Elf64SHalf = i16;
pub type Elf64Off = u64;
pub type Elf64SWord = i32;
pub type Elf64Word = u32;
pub type Elf64XWord = u64;
pub type Elf64SXWord = i64;

// Elf file types
const ET_NONE: Elf64Half = 0;
const ET_REL: Elf64Half = 1;
const ET_EXEC: Elf64Half = 2;
const ET_DYN: Elf64Half = 3;
const ET_CORE: Elf64Half = 4;

pub const EI_NIDENT: usize = 16;
// e_ident indexes
pub const EI_MAG0: usize = 0;
pub const EI_MAG1: usize = 1;
pub const EI_MAG2: usize = 2;
pub const EI_MAG3: usize = 3;
pub const EI_CLASS: usize = 4;
pub const EI_DATA: usize = 5;
pub const EI_VERSION: usize = 6;
pub const EI_OSABI: usize = 7;
pub const EI_PAD: usize = 8;

// EI_MAG
pub const ELFMAG: &str = "\x7fELF";
const ELFMAG0: u8 = 0x7f;
const ELFMAG1: u8 = b'E';
const ELFMAG2: u8 = b'L';
const ELFMAG3: u8 = b'F';
const SELFMAG: usize = 4;

// EI_CLASS
const ELFCLASSNONE: u8 = 0;
const ELFCLASS32: u8 = 1;
const ELFCLASS64: u8 = 2;
const ELFCLASSNUM: usize = 3;

// EI_DATA
const ELFDATANONE: u8 = 0;
const ELFDATA2LSB: u8 = 1;
const ELFDATA2MSB: u8 = 2;

// EI_VERSION
const EV_NONE: Elf64Half = 0;
const EV_CURRENT: Elf64Half = 1;
const EV_NUM: usize = 2;

// OS_ABI
const ELFOSABI_NONE: u8 = 0;
const ELFOSABI_LINUX: u8 = 3;

// Machine types defined in elf-em.h
const EM_NONE: Elf64Half = 0;
const EM_M32: Elf64Half = 1;
const EM_SPARC: Elf64Half = 2;
const EM_386: Elf64Half = 3;
const EM_68K: Elf64Half = 4;
const EM_88K: Elf64Half = 5;
const EM_486: Elf64Half = 6; // Perhaps disused
const EM_860: Elf64Half = 7;
const EM_MIPS: Elf64Half = 8; // MIPS R3000 (officially, big-endian only)

// Next two are historical and binaries and
// modules of these types will be rejected by
// Linux.
// const EM_MIPS_RS3_LE: Elf64Half = 10; // MIPS R3000 little-endian
// const EM_MIPS_RS4_BE: Elf64Half = 10; // MIPS R4000 big-endian */
const EM_PARISC: Elf64Half = 15; // HPPA
const EM_SPARC32PLUS: Elf64Half = 18; // Sun's "v8plus"
const EM_PPC: Elf64Half = 20; // PowerPC
const EM_PPC64: Elf64Half = 21; // PowerPC64
const EM_SPU: Elf64Half = 23; // Cell BE SPU
const EM_ARM: Elf64Half = 40; // ARM 32 bit
const EM_SH: Elf64Half = 42; // SuperH
const EM_SPARCV9: Elf64Half = 43; // SPARC v9 64-bit
const EM_H8_300: Elf64Half = 46; // Renesas H8/300
const EM_IA_64: Elf64Half = 50; // HP/Intel IA-64
const EM_X86_64: Elf64Half = 62; // AMD x86-64
const EM_S390: Elf64Half = 22; // IBM S/390
const EM_CRIS: Elf64Half = 76; // Axis Communications 32-bit embedded processor
const EM_M32R: Elf64Half = 88; // Renesas M32R
const EM_MN10300: Elf64Half = 89; // Panasonic/MEI MN10300, AM33
const EM_OPENRISC: Elf64Half = 92; // OpenRISC 32-bit embedded processor
const EM_ARCOMPACT: Elf64Half = 93; // ARCompact processor
const EM_XTENSA: Elf64Half = 94; // Tensilica Xtensa Architecture
const EM_BLACKFIN: Elf64Half = 106; // ADI Blackfin Processor
const EM_UNICORE: Elf64Half = 110; // UniCore-32
const EM_ALTERA_NIOS2: Elf64Half = 113; // Altera Nios II soft-core processor
const EM_TI_C6000: Elf64Half = 140; // TI C6X DSPs
const EM_HEXAGON: Elf64Half = 164; // QUALCOMM Hexagon
const EM_NDS32: Elf64Half = 167; // Andes Technology compact code size
                                 // embedded RISC processor family
const EM_AARCH64: Elf64Half = 183; // ARM 64 bit
const EM_TILEPRO: Elf64Half = 188; // Tilera TILEPro
const EM_MICROBLAZE: Elf64Half = 189; // Xilinx MicroBlaze
const EM_TILEGX: Elf64Half = 191; // Tilera TILE-Gx
const EM_ARCV2: Elf64Half = 195; // ARCv2 Cores
const EM_RISCV: Elf64Half = 243; // RISC-V
const EM_BPF: Elf64Half = 247; // Linux BPF - in-kernel virtual machine
const EM_CSKY: Elf64Half = 252; // C-SKY
const EM_LOONGARCH: Elf64Half = 258; // LoongArch
const EM_FRV: Elf64Half = 0x5441; // Fujitsu FR-V

pub mod types {
    use super::*;

    #[derive(PartialEq, Eq)]
    pub enum ElfMachine {
        Unknown,
        M32,
        Sparc,
        I386,
        M68k,
        M88k,
        I486,
        I860,
        Mips,
        PaRisc,
        Sparc32Plus,
        PPc,
        PPc64,
        Spu,
        Arm,
        SH,
        Sparcv9,
        H8_300,
        Ia64,
        X86_64,
        S390,
        Cris,
        M32R,
        MN10300,
        Openrisc,
        Arcompact,
        Xtensa,
        Blackfin,
        Unicore,
        AlteraNios2,
        TiC6000,
        Hexagon,
        Nds32,
        Aarch64,
        TilePro,
        MicroBlaze,
        TileGx,
        ArcV2,
        RiscV,
        Bpf,
        CSky,
        LoongArch,
        FrV,
    }

    impl From<Elf64Half> for ElfMachine {
        fn from(value: Elf64Half) -> Self {
            match value {
                EM_M32 => Self::M32,
                EM_SPARC => Self::Sparc,
                EM_386 => Self::I386,
                EM_68K => Self::M68k,
                EM_88K => Self::M88k,
                EM_486 => Self::I486,
                EM_860 => Self::I860,
                EM_MIPS => Self::Mips,
                EM_PARISC => Self::PaRisc,
                EM_SPARC32PLUS => Self::Sparc32Plus,
                EM_PPC => Self::PPc,
                EM_PPC64 => Self::PPc64,
                EM_SPU => Self::Spu,
                EM_ARM => Self::Arm,
                EM_SH => Self::SH,
                EM_SPARCV9 => Self::Sparcv9,
                EM_H8_300 => Self::H8_300,
                EM_IA_64 => Self::Ia64,
                EM_X86_64 => Self::X86_64,
                EM_S390 => Self::S390,
                EM_CRIS => Self::Cris,
                EM_M32R => Self::M32R,
                EM_MN10300 => Self::MN10300,
                EM_OPENRISC => Self::Openrisc,
                EM_ARCOMPACT => Self::Arcompact,
                EM_XTENSA => Self::Xtensa,
                EM_BLACKFIN => Self::Blackfin,
                EM_UNICORE => Self::Unicore,
                EM_ALTERA_NIOS2 => Self::AlteraNios2,
                EM_TI_C6000 => Self::TiC6000,
                EM_HEXAGON => Self::Hexagon,
                EM_NDS32 => Self::Nds32,
                EM_AARCH64 => Self::Aarch64,
                EM_TILEPRO => Self::TilePro,
                EM_MICROBLAZE => Self::MicroBlaze,
                EM_TILEGX => Self::TileGx,
                EM_ARCV2 => Self::ArcV2,
                EM_RISCV => Self::RiscV,
                EM_BPF => Self::Bpf,
                EM_CSKY => Self::CSky,
                EM_LOONGARCH => Self::LoongArch,
                EM_FRV => Self::FrV,
                _ => Self::Unknown,
            }
        }
    }

    #[derive(PartialEq, Eq)]
    pub enum ElfClass {
        Class32,
        Class64,
        Unknown,
    }

    impl From<u8> for ElfClass {
        fn from(value: u8) -> Self {
            match value {
                ELFCLASS32 => Self::Class32,
                ELFCLASS64 => Self::Class64,
                _ => Self::Unknown,
            }
        }
    }

    #[derive(PartialEq, Eq)]
    pub enum ElfDataLayout {
        Lsb,
        Msb,
        Unknown,
    }

    impl From<u8> for ElfDataLayout {
        fn from(value: u8) -> Self {
            match value {
                ELFDATA2LSB => Self::Lsb,
                ELFDATA2MSB => Self::Msb,
                _ => Self::Unknown,
            }
        }
    }

    #[derive(PartialEq, Eq)]
    pub enum ElfType {
        Unknown,
        Relocatable,
        Executable,
        Dynamic,
        Core,
    }

    impl From<Elf64Half> for ElfType {
        fn from(value: Elf64Half) -> Self {
            match value {
                ET_REL => Self::Relocatable,
                ET_EXEC => Self::Executable,
                ET_DYN => Self::Dynamic,
                ET_CORE => Self::Core,
                _ => Self::Unknown,
            }
        }
    }

    #[derive(PartialEq, Eq)]
    pub enum ElfVersion {
        Unknown,
        Current,
    }

    impl From<Elf64Half> for ElfVersion {
        fn from(value: Elf64Half) -> Self {
            match value {
                EV_CURRENT => Self::Current,
                _ => Self::Unknown,
            }
        }
    }

    #[derive(PartialEq, Eq)]
    pub enum ElfSegmentType {
        Null,
        Load,
        Dynamic,
        Interp,
        Note,
        ShLib,
        Phdr,
        Tls,
        Unknown,
    }

    impl From<Elf64Word> for ElfSegmentType {
        fn from(value: Elf64Word) -> Self {
            match value {
                0 => Self::Null,
                1 => Self::Load,
                2 => Self::Dynamic,
                3 => Self::Interp,
                4 => Self::Note,
                5 => Self::ShLib,
                6 => Self::Phdr,
                7 => Self::Tls,
                _ => Self::Unknown,
            }
        }
    }
}
