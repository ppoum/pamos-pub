use crate::{protocol::RawSimpleTextOutputProtocol, CStr16, Status};

#[repr(transparent)]
pub struct Output(RawSimpleTextOutputProtocol);

impl Output {
    pub fn write(&mut self, s: &CStr16) -> Status {
        unsafe { (self.0.output_string)(&mut self.0, s.as_ptr()) }
    }
}
