use core::slice;

#[repr(transparent)]
pub struct CStr16([u16]);

impl CStr16 {
    /// Converts a u16 slice to a &CStr16.
    /// # Safety
    /// This function assumes the slice is null-terminated and that
    /// every character is a valid UCS-2 character. Failure to match the
    /// above conditions may result in undefined behaviour.
    pub const unsafe fn from_u16_unsafe(chars: &[u16]) -> &Self {
        &*(chars as *const [u16] as *const Self)
    }

    /// Converts a pointer to a u16 array to a &CStr16.
    /// # Safety
    /// The pointer should point to a valid memory location that contains a valid
    /// UCS-2 string. Failure to due so may result in the string being arbitrarily long
    /// and/or have invalid characters.
    pub unsafe fn from_ptr<'a>(ptr: *const u16) -> &'a Self {
        let mut length = 0;
        while *ptr.add(length) != 0 {
            length += 1;
        }

        Self::from_u16_unsafe(slice::from_raw_parts(ptr, length))
    }

    pub const fn as_ptr(&self) -> *const u16 {
        self.0.as_ptr()
    }
}
