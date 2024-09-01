use core::{
    ffi::c_void,
    fmt::{self, Write},
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::println;

use super::{boot_services::BootServices, protocols::Output, status::EfiResult, SystemTable};

pub static _ST: AtomicPtr<SystemTable> = AtomicPtr::new(ptr::null_mut());

#[panic_handler]
fn _panic_handler(panic_info: &core::panic::PanicInfo) -> ! {
    // NOTE: PanicInfo#payload isn't created in core, since it requires allocation.
    //
    if _st_is_set() {
        println!("panic occurred: {:?}", panic_info);
        // FIXME: PanicInfo#message is getting stabilized in 1.81

        // if let Some(msg) = panic_info.message() {
        //     println!("panic occurred: {}", msg);
        // } else {
        //     println!("panic occurred");
        // }
    }
    loop {}
}

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
        stdout.write_fmt(format_args!("{}\r\n", args))
    } else {
        stdout.write_fmt(args)
    }
    .expect("error writing to output")
}

/// A wrapper around an allocated pool pointer. Frees the pool once the object goes out of scope.
/// # Safety
/// It is assumed that the pool is valid as long as this object exists. The data should be
/// initialized when the `AllocatedPool` object is created.
pub struct AllocatedPool<T: ?Sized> {
    _marker: core::marker::PhantomData<T>,
    boot_services: BootServices,
    ptr: *mut c_void,
    slice_size: Option<usize>,
}

impl<T> AllocatedPool<T> {
    pub fn try_new(boot_services: BootServices) -> EfiResult<Self> {
        let len = size_of::<T>();
        let ptr = boot_services.allocate_pool(len)?;

        Ok(Self {
            _marker: core::marker::PhantomData,
            boot_services,
            ptr,
            slice_size: None,
        })
    }
}

impl<T> AsRef<T> for AllocatedPool<T> {
    fn as_ref(&self) -> &T {
        unsafe { &*(self.ptr as *const T) }
    }
}

impl<T> AsMut<T> for AllocatedPool<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.ptr as *mut T) }
    }
}

impl<T> AllocatedPool<[T]> {
    pub fn try_new(boot_services: BootServices, len: usize) -> EfiResult<Self> {
        let i = size_of::<T>();
        let ptr = boot_services.allocate_pool(i * len)?;

        Ok(Self {
            _marker: core::marker::PhantomData,
            boot_services,
            ptr,
            slice_size: Some(len),
        })
    }
}

impl<T> AsRef<[T]> for AllocatedPool<[T]> {
    fn as_ref(&self) -> &[T] {
        // Safety: The size of the slice is known, we expect it to have been initialized with
        // proper `T` data.
        unsafe {
            core::slice::from_raw_parts(
                self.ptr as *const T,
                self.slice_size.expect(
                    "AllocatedPool containing a slice did not have a length specified (weird)",
                ),
            )
        }
    }
}

impl<T> AsMut<[T]> for AllocatedPool<[T]> {
    fn as_mut(&mut self) -> &mut [T] {
        // Safety: The size of the slice is known, we expect it to have been initialized with
        // proper `T` data.
        unsafe {
            core::slice::from_raw_parts_mut(
                self.ptr as *mut T,
                self.slice_size.expect(
                    "AllocatedPool containing a slice did not have a length specified (weird)",
                ),
            )
        }
    }
}

impl<T: ?Sized> Drop for AllocatedPool<T> {
    fn drop(&mut self) {
        let _ = self.boot_services.free_pool(self.ptr);
    }
}
