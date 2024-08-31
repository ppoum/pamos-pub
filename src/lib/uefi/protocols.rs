mod console;
mod loaded_image;

pub use console::*;
pub use loaded_image::*;

use super::{boot_services::BootServices, status::StatusError, Guid, Handle};

pub enum ProtocolLocateError {
    Unsupported,
    Error(StatusError),
}

pub trait RawProtocol: Sized {
    const GUID: Guid;

    fn try_locate_protocol(
        boot_services: &BootServices,
        handle: Handle,
    ) -> Result<*const Self, ProtocolLocateError> {
        let res = boot_services.generic_handle_protocol(handle, &Self::GUID);
        let void_interface = match res {
            Ok(Some(x)) => x,
            Ok(None) => return Err(ProtocolLocateError::Unsupported),
            Err(e) => return Err(ProtocolLocateError::Error(e)),
        };

        Ok(void_interface as *const Self)
    }
}

pub trait Protocol {
    fn try_locate(
        handle: Handle,
        boot_services: &BootServices,
    ) -> Result<&Self, ProtocolLocateError>;
}
