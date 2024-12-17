use bitfield::bitfield;

use crate::uefi::{boot_services::BootServices, AllocateType, PhysicalAddress, VirtualAddress};

pub type Page4Table = Pml4;
pub type Page3Table = PdpTable;
pub type Page2Table = PageDirectory;
pub type Page1Table = PageTable;

pub type Page4Entry = Pml4Entry;
pub type Page3Entry = PdptEntry;
pub type Page2Entry = PageDirectoryEntry;
pub type Page1Entry = PageTableEntry;

#[repr(C, align(4096))]
pub struct Pml4 {
    entries: [Pml4Entry; 512],
}

impl Pml4 {
    /// Allocates a new PML4 table and fill it out with 0s
    pub fn new_allocate_empty(boot_services: BootServices) -> &'static mut Self {
        let pml4_page_base = boot_services
            .leaky_allocate_pages(AllocateType::MaxAddress, 1, Some(0x80000))
            .unwrap();
        let pml4 = pml4_page_base as *mut Self;

        // SAFETY: Struct has been properly allocated & initialized
        unsafe {
            pml4.write_bytes(0, 1);
            pml4.as_mut().unwrap()
        }
    }

    pub fn get_entry(&self, addr: VirtualAddress) -> Option<&Pml4Entry> {
        // Get bits 47 - 39
        let idx = (addr >> (9 * 3 + 12)) & (2u64.pow(9) - 1);
        let entry = &self.entries[idx as usize];
        if entry.present() == 0 {
            None
        } else {
            Some(entry)
        }
    }
}

#[repr(C, align(4096))]
pub struct PdpTable {
    entries: [PdptEntry; 512],
}

impl PdpTable {
    pub fn new_allocate_empty(boot_services: BootServices) -> &'static mut Self {
        let pdpt_page_base = boot_services
            .leaky_allocate_pages(AllocateType::MaxAddress, 1, Some(0x80000))
            .unwrap();
        let pdpt = pdpt_page_base as *mut Self;

        // SAFETY: Struct has been properly allocated & initialized
        unsafe {
            pdpt.write_bytes(0, 1);
            pdpt.as_mut().unwrap()
        }
    }

    /// # Safety
    /// Assumes that the entry contains the address of a valid P3 table.
    pub unsafe fn from_pml4_entry(entry: &Pml4Entry) -> &'static mut Self {
        debug_assert!(entry.present() != 0);
        let addr = entry.addr() << 12;
        (addr as *mut Self).as_mut().unwrap()
    }

    pub fn get_entry(&self, addr: VirtualAddress) -> Option<&PdptEntry> {
        // Get bits 38 - 30
        let idx = (addr >> (9 * 2 + 12)) & (2u64.pow(9) - 1);
        let entry = &self.entries[idx as usize];
        if entry.present() == 0 {
            None
        } else {
            Some(entry)
        }
    }
}

#[repr(C, align(4096))]
pub struct PageDirectory {
    entries: [PageDirectoryEntry; 512],
}

impl PageDirectory {
    pub fn new_allocate_empty(boot_services: BootServices) -> &'static mut Self {
        let pd_page_base = boot_services
            .leaky_allocate_pages(AllocateType::MaxAddress, 1, Some(0x80000))
            .unwrap();
        let pd = pd_page_base as *mut Self;

        // SAFETY: Struct has been properly allocated & initialized
        unsafe {
            pd.write_bytes(0, 1);
            pd.as_mut().unwrap()
        }
    }
    /// # Safety
    /// Assumes that the entry contains the address of a valid P2 table.
    pub unsafe fn from_pdpt_entry(entry: &PdptEntry) -> &'static mut Self {
        debug_assert!(entry.present() != 0);
        debug_assert!(entry.huge_page() == 0);
        let addr = entry.addr() << 12;
        (addr as *mut Self).as_mut().unwrap()
    }

    pub fn get_entry(&self, addr: VirtualAddress) -> Option<&PageDirectoryEntry> {
        // Get bits 29 - 21
        let idx = (addr >> (9 + 12)) & (2u64.pow(9) - 1);
        let entry = &self.entries[idx as usize];
        if entry.present() == 0 {
            None
        } else {
            Some(entry)
        }
    }
}

#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

impl PageTable {
    pub fn new_allocate_empty(boot_services: BootServices) -> &'static mut Self {
        let pt_page_base = boot_services
            .leaky_allocate_pages(AllocateType::MaxAddress, 1, Some(0x80000))
            .unwrap();
        let pt = pt_page_base as *mut Self;

        // SAFETY: Struct has been properly allocated & initialized
        unsafe {
            pt.write_bytes(0, 1);
            pt.as_mut().unwrap()
        }
    }
    /// # Safety
    /// Assumes that the entry contains the address of a valid P2 table.
    pub unsafe fn from_page_directory_entry(entry: &PageDirectoryEntry) -> &'static mut Self {
        debug_assert!(entry.present() != 0);
        debug_assert!(entry.huge_page() == 0);
        let addr = entry.addr() << 12;
        (addr as *mut Self).as_mut().unwrap()
    }

    pub fn get_entry(&self, addr: VirtualAddress) -> Option<&PageTableEntry> {
        // Get bits 20 - 12
        let idx = (addr >> 12) & (2u64.pow(9) - 1);
        let entry = &self.entries[idx as usize];
        if entry.present() == 0 {
            None
        } else {
            Some(entry)
        }
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct Pml4Entry(u64);
    nx, set_nx: 63, 63;
    avl_1, _: 62, 52;
    addr, set_addr: 47, 12;
    avl_2, _: 11, 8;
    avl_3, _: 6, 6;
    accessed, _: 5, 5;
    cache_disable, _: 4, 4;
    write_through, _: 3, 3;
    user_accessible, set_user_accessible: 2, 2;
    rw, set_rw: 1, 1;
    present, set_present: 0, 0;
}

impl Pml4Entry {
    pub fn not_present() -> Self {
        Self(0)
    }

    pub fn from_addr(addr: u64) -> Self {
        assert!(addr < 2u64.pow(48));
        assert!(addr % 4096 == 0);
        let mut x = Self(0);
        x.set_addr(addr >> 12);
        x.set_rw(1);
        x.set_present(1);
        x
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct PdptEntry(u64);
    nx, set_nx: 63, 63;
    avl_1, _: 62, 52;
    addr, set_addr: 47, 12;
    avl_2, _: 11, 8;
    huge_page, _: 7, 7;
    avl_3, _: 6, 6;
    accessed, _: 5, 5;
    cache_disable, _: 4, 4;
    write_through, _: 3, 3;
    user_accessible, set_user_accessible: 2, 2;
    rw, set_rw: 1, 1;
    present, set_present: 0, 0;
}

impl PdptEntry {
    pub fn not_present() -> Self {
        Self(0)
    }

    pub fn from_addr(addr: u64) -> Self {
        assert!(addr < 2u64.pow(48));
        assert!(addr % 4096 == 0);
        let mut x = Self(0);
        x.set_addr(addr >> 12);
        x.set_rw(1);
        x.set_present(1);
        x
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct PageDirectoryEntry(u64);
    nx, set_nx: 63, 63;
    avl_1, _: 62, 52;
    addr, set_addr: 47, 12;
    avl_2, _: 11, 8;
    huge_page, _: 7, 7;
    avl_3, _: 6, 6;
    accessed, _: 5, 5;
    cache_disable, _: 4, 4;
    write_through, _: 3, 3;
    user_accessible, set_user_accessible: 2, 2;
    rw, set_rw: 1, 1;
    present, set_present: 0, 0;
}

impl PageDirectoryEntry {
    pub fn not_present() -> Self {
        Self(0)
    }

    pub fn from_addr(addr: u64) -> Self {
        assert!(addr < 2u64.pow(48));
        assert!(addr % 4096 == 0);
        let mut x = Self(0);
        x.set_addr(addr >> 12);
        x.set_rw(1);
        x.set_present(1);
        x
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct PageTableEntry(u64);
    nx, set_nx: 63, 63;
    protection_key, _: 62, 59;
    avl_1, _: 58, 52;
    addr, set_addr: 47, 12;
    avl_2, _: 11, 9;
    global, _: 8, 8;
    pat, _: 7, 7;
    dirty, _: 6, 6;
    accessed, _: 5, 5;
    cache_disable, _: 4, 4;
    write_through, _: 3, 3;
    user_accessible, set_user_accessible: 2, 2;
    rw, set_rw: 1, 1;
    present, set_present: 0, 0;

}

impl PageTableEntry {
    pub fn not_present() -> Self {
        Self(0)
    }

    pub fn from_addr(addr: u64) -> Self {
        assert!(addr < 2u64.pow(48));
        assert!(addr % 4096 == 0);
        let mut x = Self(0);
        x.set_addr(addr >> 12);
        // FIXME: Don't make every page writable
        x.set_rw(1);
        x.set_present(1);
        x
    }
}

/// For now, ignore huge pages and only create "regular" pages
pub fn new_page_map(
    boot_services: BootServices,
    pml4: &mut Pml4,
    v_addr: VirtualAddress,
    p_addr: PhysicalAddress,
) {
    let p1_idx = ((v_addr >> 12) & (2u64.pow(9) - 1)) as usize;
    let p2_idx = ((v_addr >> (12 + 9)) & (2u64.pow(9) - 1)) as usize;
    let p3_idx = ((v_addr >> (12 + 9 * 2)) & (2u64.pow(9) - 1)) as usize;
    let p4_idx = ((v_addr >> (12 + 9 * 3)) & (2u64.pow(9) - 1)) as usize;

    let p3_table = {
        let entry = &pml4.entries[p4_idx];
        if entry.present() != 0 {
            unsafe { PdpTable::from_pml4_entry(entry) }
        } else {
            // Create a new P3 table and P4 entry
            let p3 = PdpTable::new_allocate_empty(boot_services);
            let p3_addr = (p3 as *const _) as usize;
            let entry = Pml4Entry::from_addr(p3_addr as u64);
            pml4.entries[p4_idx] = entry;
            p3
        }
    };

    let p2_table = {
        let entry = &p3_table.entries[p3_idx];
        if entry.present() != 0 {
            unsafe { PageDirectory::from_pdpt_entry(entry) }
        } else {
            // Create a new P2 table and P3 entry
            let p2 = PageDirectory::new_allocate_empty(boot_services);
            let p2_addr = (p2 as *const _) as usize;
            let entry = PdptEntry::from_addr(p2_addr as u64);
            p3_table.entries[p3_idx] = entry;
            p2
        }
    };

    let p1_table = {
        let entry = &p2_table.entries[p2_idx];
        if entry.present() != 0 {
            unsafe { PageTable::from_page_directory_entry(entry) }
        } else {
            // Create a new P1 table and P2 entry
            let p1 = PageTable::new_allocate_empty(boot_services);
            let p1_addr = (p1 as *const _) as usize;
            let entry = PageDirectoryEntry::from_addr(p1_addr as u64);
            p2_table.entries[p2_idx] = entry;
            p1
        }
    };

    let entry = PageTableEntry::from_addr(p_addr);
    p1_table.entries[p1_idx] = entry;
}

pub fn map_range(
    boot_services: BootServices,
    pml4: &mut Pml4,
    v_addr: VirtualAddress,
    p_addr: PhysicalAddress,
    page_count: u64,
) {
    for i in 0..page_count {
        let v = v_addr + 0x1000 * i;
        let p = p_addr + 0x1000 * i;
        new_page_map(boot_services, pml4, v, p);
    }
}

pub fn translate_virtual_to_physical(pml4: &Pml4, addr: VirtualAddress) -> Option<PhysicalAddress> {
    let entry = pml4.get_entry(addr)?;

    // SAFETY: Our P4 entry must be valid
    let p3 = unsafe { PdpTable::from_pml4_entry(entry) };
    let entry = p3.get_entry(addr)?;

    if entry.huge_page() != 0 {
        // 1GB page
        let page_base = entry.addr() << 30;
        let offset = addr % (2u64.pow(30));
        return Some(page_base + offset);
    }

    // SAFETY: ^
    let p2 = unsafe { PageDirectory::from_pdpt_entry(entry) };
    let entry = p2.get_entry(addr)?;

    if entry.huge_page() != 0 {
        // 2MB page
        let page_base = entry.addr() << 21;
        let offset = addr % (2u64.pow(21));
        return Some(page_base + offset);
    }

    // SAFETY: ^^
    let p1 = unsafe { PageTable::from_page_directory_entry(entry) };
    let entry = p1.get_entry(addr)?;

    let page_base = entry.addr() << 12;
    let offset = addr % 4096;

    Some(page_base + offset)
}
