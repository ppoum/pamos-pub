#[macro_export]
macro_rules! cstr16 {
    () => {{
        const SLICE: &[u16] = &[0];
        // Safety: Always safe (empty string, ends with null)
        unsafe { $crate::uefi::CStr16::from_u16_unsafe(SLICE) }
    }};
    ($s:literal) => {{
        const SLICE: &[u16] = &uefi_macros::ucs2_slice!($s);
        // Safety: ucs2_slice macro always creates a valid UCS2 string slice
        unsafe { $crate::uefi::string::CStr16::from_u16_unsafe(SLICE) }
    }};
}

#[macro_export]
macro_rules! guid {
    ($s:literal) => {{
        const ARRAY: [u8; 16] = uefi_macros::guid_str_to_bytes!($s);
        $crate::uefi::Guid::from_bytes(ARRAY)
    }};
}

#[macro_export]
macro_rules! print {
    ($s:literal) => {
        unsafe {
            $crate::uefi::helper::_get_st_panicking()
                .stdout()
                .write($crate::cstr16!($s));
        }
    };
    ($($arg:tt)*) => {{
        let stdout = unsafe { $crate::uefi::helper::_get_st_panicking().stdout() };
        $crate::uefi::helper::_print(core::format_args!($($arg)*), stdout, true);
    }};
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {{
        let stdout = unsafe { $crate::uefi::helper::_get_st_panicking().stdout() };
        $crate::uefi::helper::_print(core::format_args!($($arg)*), stdout, true);
    }};
}
