pub type EfiResult<T> = Result<T, StatusError>;

#[repr(transparent)]
pub struct Status(usize);

const UPPER_BIT_MASK: usize = 1 << (usize::BITS - 1);
impl Status {
    pub fn ok() -> Self {
        Self(0)
    }

    pub fn to_result(self) -> Result<(), StatusError> {
        if self.0 & UPPER_BIT_MASK == 0 {
            Ok(())
        } else {
            todo!()
        }
    }
}

impl From<usize> for Status {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

pub enum StatusError {
    LoadError = 1,
    InvalidParameter,
    Unsupported,
    BadBufferSize,
    BufferTooSmall,
    NotReady,
    DeviceError,
    WriteProtected,
    OutOfResources,
    VolumeCorrupted,
    VolumeFull,
    NoMedia,
    MediaChanged,
    NotFound,
    AccessDenied,
    NoResponse,
    NoMapping,
    Timeout,
    NotStarted,
    AlreadyStarted,
    Aborted,
    IcmpError,
    TftpError,
    ProtocolError,
    IncompatibleVersion,
    SecurityViolation,
    CrcError,
    EndOfMedia,
    EndOfFile,
    InvalidLanguage,
    CompromisedData,
    IpAddressConflict,
    HttpError,

    // Currently, warnings also get mapped to `Unknown`, might want to handle those status codes
    Unknown,
}

impl From<usize> for StatusError {
    fn from(value: usize) -> Self {
        // Unset upper bit
        let value = value & !UPPER_BIT_MASK;
        match value {
            1 => Self::LoadError,
            2 => Self::InvalidParameter,
            3 => Self::Unsupported,
            4 => Self::BadBufferSize,
            5 => Self::BufferTooSmall,
            6 => Self::NotReady,
            7 => Self::DeviceError,
            8 => Self::WriteProtected,
            9 => Self::OutOfResources,
            10 => Self::VolumeCorrupted,
            11 => Self::VolumeFull,
            12 => Self::NoMedia,
            13 => Self::MediaChanged,
            14 => Self::NotFound,
            15 => Self::AccessDenied,
            16 => Self::NoResponse,
            17 => Self::NoMapping,
            18 => Self::Timeout,
            19 => Self::NotStarted,
            20 => Self::AlreadyStarted,
            21 => Self::Aborted,
            22 => Self::IcmpError,
            23 => Self::TftpError,
            24 => Self::ProtocolError,
            25 => Self::IncompatibleVersion,
            26 => Self::SecurityViolation,
            27 => Self::CrcError,
            28 => Self::EndOfMedia,
            29 => Self::EndOfFile,
            30 => Self::InvalidLanguage,
            31 => Self::CompromisedData,
            32 => Self::IpAddressConflict,
            33 => Self::HttpError,

            _ => Self::Unknown,
        }
    }
}
