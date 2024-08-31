use super::{protocol::RawSimpleTextOutputProtocol, status::Status, CStr16};

#[repr(transparent)]
pub struct Output(RawSimpleTextOutputProtocol);

impl Output {
    pub fn write(&mut self, s: &CStr16) -> Status {
        unsafe { (self.0.output_string)(&mut self.0, s.as_ptr()) }
    }
}
