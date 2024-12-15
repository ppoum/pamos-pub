use core::ptr;

pub struct BootInformationWriter {
    base: *mut u8,
    max_len: usize,
    index: usize,
}

impl BootInformationWriter {
    /// * `base`: Base address
    /// * `len`: Allocated size (in bytes)
    /// # Safety
    /// The address range from `base` to `(base+len)` must be fully allocated. If it is not, memory may
    /// be overwritten.
    pub unsafe fn new(base: *mut u8, len: usize) -> Self {
        // Align the base to 8-byte
        let offset = base.align_offset(8);
        if offset == usize::MAX {
            panic!("Error trying to find BootInformation base offset");
        }
        if offset >= len {
            panic!("BootInformation base address must be 8-byte aligned (out of memory trying to align)");
        }
        let base = base.add(offset);
        let len = len - offset;

        let mut s = Self {
            base,
            max_len: len,
            index: 0,
        };

        // Write header
        s.write_u32(0); // total_size, replace when closing the Writer
        s.write_u32(0); // reserved (always 0)

        s
    }

    pub fn close(mut self) -> *mut u8 {
        // Write closing tag (type 0, size 8)
        self.write_u32(0);
        self.write_u32(8);

        // Overwrite header to fix the total_size value
        let written_bytes = self.index;
        self.index = 0;
        self.write_u32(written_bytes as u32);

        self.base
    }

    //
    // Internals
    // Safety: By checking the index first, we can ensure we are writing to allocated memory. The
    // burden of safety is delegated to the constructor of the struct.
    //

    fn write_u32(&mut self, n: u32) {
        if (self.max_len - self.index) < 4 {
            panic!("Tried to write MB2 BootInformation, but the buffer was full.");
        }
        unsafe {
            ptr::copy_nonoverlapping(n.to_le_bytes().as_ptr(), self.base.add(self.index), 4);
        }
    }
}
