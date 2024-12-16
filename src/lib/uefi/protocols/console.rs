use core::{
    fmt::{self, Write},
    ptr,
};

use uefi_macros::Protocol;

use crate::{
    guid,
    uefi::{
        status::{EfiResult, Status},
        string::CStr16,
        PhysicalAddress,
    },
};

use super::RawProtocol;

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

#[repr(transparent)]
#[derive(Protocol)]
pub struct GraphicsOutputProtocol(RawGraphicsOutputProtocol);
impl GraphicsOutputProtocol {
    pub fn mode(&self) -> Option<&ProtocolMode> {
        // SAFETY: Pointer is valid, guaranteed by UEFI spec
        unsafe { self.0.mode.as_ref() }
    }

    pub fn query_mode(&self, mode_number: u32) -> EfiResult<&GraphicsOutputModeInfo> {
        let mut info: *mut GraphicsOutputModeInfo = ptr::null_mut();
        let info_ptr = &mut info as *mut _;
        let mut info_size = 0usize;
        // SAFETY: Guaranteed by UEFI spec
        unsafe {
            (self.0.query_mode)(
                &self.0 as *const _ as *mut _,
                mode_number,
                &mut info_size as *mut _,
                info_ptr,
            )
            .to_result()?;
            Ok(info.as_ref().unwrap())
        }
    }

    pub fn set_mode(&self, mode_number: u32) -> EfiResult<()> {
        // SAFETY: Guaranteed by UEFI spec
        unsafe { (self.0.set_mode)(&self.0 as *const _ as *mut _, mode_number).to_result() }
    }
}

#[repr(C)]
pub struct RawGraphicsOutputProtocol {
    pub query_mode: unsafe extern "efiapi" fn(
        this: *mut Self,
        mode_number: u32,
        info_size: *mut usize,
        info: *mut *mut GraphicsOutputModeInfo,
    ) -> Status,
    pub set_mode: unsafe extern "efiapi" fn(*mut Self, mode_number: u32) -> Status,
    /// `blt_buffer` and `delta` are optional
    pub blt: unsafe extern "efiapi" fn(
        *mut Self,
        blt_buffer: *mut BltPixel,
        blt_operation: BltOperation,
        source_x: usize,
        source_y: usize,
        dest_x: usize,
        dest_y: usize,
        width: usize,
        height: usize,
        delta: usize,
    ) -> Status,
    pub mode: *mut ProtocolMode,
}

impl RawProtocol for RawGraphicsOutputProtocol {
    const GUID: crate::uefi::Guid = guid!("9042A9DE-23DC-4A38-96FB-7ADED080516A");
}

#[repr(C)]
#[derive(Debug)]
pub enum PixelFormat {
    RedGreenBlueReserved8BitPerColor,
    BlueGreenRedReserved8BitPerColor,
    BitMask,
    BltOnly,
    _MAX,
}

#[repr(C)]
#[derive(Debug)]
pub struct PixelBitmask {
    red_mask: u32,
    green_mask: u32,
    blue_mask: u32,
    _reserved_mask: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct GraphicsOutputModeInfo {
    pub version: u32,
    pub horizontal_res: u32,
    pub vertical_res: u32,
    pub pixel_format: PixelFormat,
    pub pixel_info: PixelBitmask,
    pub pixels_per_scanline: u32,
}

#[repr(C)]
pub struct BltPixel {
    blue: u8,
    green: u8,
    red: u8,
    _reserved: u8,
}

#[repr(C)]
pub enum BltOperation {
    VideoFill,
    VideoToBltBuffer,
    BufferToVideo,
    VideoToVideo,
    _MAX,
}

#[repr(C)]
#[derive(Debug)]
pub struct ProtocolMode {
    pub max_mode: u32,
    pub mode: u32,
    pub info: *mut GraphicsOutputModeInfo,
    pub info_size: usize,
    pub fb_base: PhysicalAddress,
    pub fb_size: usize,
}
