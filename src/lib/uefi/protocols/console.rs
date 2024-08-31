use crate::uefi::{status::Status, CStr16};

pub type Output = SimpleTextOutputProtocol;

#[repr(transparent)]
pub struct SimpleTextOutputProtocol(RawSimpleTextOutputProtocol);

impl SimpleTextOutputProtocol {
    pub fn write(&mut self, s: &CStr16) -> Status {
        unsafe { (self.0.output_string)(&mut self.0, s.as_ptr()) }
    }
}

#[repr(C)]
pub(crate) struct RawSimpleTextOutputProtocol {
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
