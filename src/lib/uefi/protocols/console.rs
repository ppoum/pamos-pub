use core::fmt::{self, Write};

use crate::uefi::{
    status::{EfiResult, Status},
    string::CStr16,
};

pub type Output = SimpleTextOutputProtocol;

#[repr(transparent)]
pub struct SimpleTextOutputProtocol(RawSimpleTextOutputProtocol);

impl SimpleTextOutputProtocol {
    pub fn write(&mut self, s: &CStr16) -> EfiResult<()> {
        unsafe { (self.0.output_string)(&mut self.0, s.as_ptr()) }.to_result()
    }
}

impl Write for SimpleTextOutputProtocol {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // Convert string from u8 bytes to u16 UCS-2
        const BUF_SIZE: usize = 256;
        let mut buffer = [0_u16; BUF_SIZE];
        let mut utf16_buf = [0_u16; 2];
        let mut i = 0;

        for char in s.chars() {
            if i == BUF_SIZE - 2 {
                // Flush buffer
                // Safety: Buffer only contains UCS-2 characters and always
                //         ends with a null byte
                let str = unsafe { CStr16::from_u16_unsafe(&buffer) };
                self.write(str).map_err(|_| fmt::Error)?;
                buffer = [0_u16; BUF_SIZE];
                i = 0;
            }

            let bytes = char.encode_utf16(&mut utf16_buf);
            let byte = if bytes.len() == 1 {
                bytes[0]
            } else {
                return Err(fmt::Error);
            };

            buffer[i] = byte;
            i += 1;
        }

        // Flush remaining data
        // Safety: Buffer only contains UCS-2 characters and always
        //         ends with a null byte
        let str = unsafe { CStr16::from_u16_unsafe(&buffer) };
        self.write(str).map_err(|_| fmt::Error)?;
        Ok(())
    }
}

#[repr(C)]
pub struct RawSimpleTextOutputProtocol {
    pub reset: unsafe extern "efiapi" fn(this: *mut Self, extended_verification: bool) -> Status,
    pub output_string: unsafe extern "efiapi" fn(this: *mut Self, string: *const u16) -> Status,
    pub test_string: unsafe extern "efiapi" fn(this: *mut Self, string: *const u16) -> Status,
    pub query_mode: unsafe extern "efiapi" fn(
        this: *mut Self,
        mode_number: usize,
        colums: *mut usize,
        rows: *mut usize,
    ) -> Status,
    pub set_mode: unsafe extern "efiapi" fn(this: *mut Self, mode_num: usize) -> Status,
    pub set_attribute: unsafe extern "efiapi" fn(this: *mut Self, attr: usize) -> Status,
    pub clear_screen: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
    pub set_cursor_position:
        unsafe extern "efiapi" fn(this: *mut Self, col: usize, row: usize) -> Status,
    pub enable_cursor: unsafe extern "efiapi" fn(this: *mut Self, visible: bool) -> Status,
    mode: usize, // TODO
}
