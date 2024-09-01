use core::{
    fmt::{self, Write},
    ptr,
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

pub fn _st_is_set() -> bool {
    _get_st_safe().is_some()
}

pub fn _print(args: fmt::Arguments, stdout: &mut Output, newline: bool) {
    if newline {
        stdout.write_fmt(format_args!("{}\n", args))
    } else {
        stdout.write_fmt(args)
    }
    .expect("error writing to output")
}
