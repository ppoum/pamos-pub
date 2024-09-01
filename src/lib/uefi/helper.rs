use core::{
    fmt, ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::uefi::string::CStr16;

use super::{protocols::Output, SystemTable};

pub static _ST: AtomicPtr<SystemTable> = AtomicPtr::new(ptr::null_mut());

pub fn register_services(st: &SystemTable) {
    _ST.store(st as *const _ as *mut _, Ordering::Relaxed);
}

/// # Safety
/// None, will panick if _ST hasn't been set to a valid SystemTable
pub unsafe fn _get_st_panicking<'a>() -> &'a mut SystemTable {
    let ptr = _ST.load(Ordering::Relaxed);
    ptr.as_mut().unwrap()
}

pub fn _get_st_safe<'a>() -> Option<&'a mut SystemTable> {
    let ptr = _ST.load(Ordering::Relaxed);
    unsafe { ptr.as_mut() }
}

pub fn _print(args: fmt::Arguments, stdout: &mut Output) {
    let str = args
        .as_str()
        .expect("String cannot be formatted at compile-time");

    // Convert string from u8 bytes to u16 UCS-2
    const BUF_SIZE: usize = 256;
    let mut buffer = [0_u16; BUF_SIZE];
    let mut utf16_buf = [0_u16; 2];
    let mut i = 0;

    for char in str.chars() {
        if i == BUF_SIZE - 1 {
            // Flush buffer
            // Safety: Buffer only contains UCS-2 characters and always
            //         ends with a null byte
            let str = unsafe { CStr16::from_u16_unsafe(&buffer) };
            stdout.write(str);
            buffer = [0_u16; BUF_SIZE];
            i = 0;
        }

        let bytes = char.encode_utf16(&mut utf16_buf);
        let byte = if bytes.len() == 1 {
            bytes[0]
        } else {
            panic!("Tried printing an invalid UCS-2 character");
        };

        buffer[i] = byte;
        i += 1;
    }

    // Flush remaining data
    // Safety: Buffer only contains UCS-2 characters and always
    //         ends with a null byte
    let str = unsafe { CStr16::from_u16_unsafe(&buffer) };
    stdout.write(str);
}
