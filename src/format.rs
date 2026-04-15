use std::{
    ffi::{c_void, CStr, CString},
    fmt,
    os::raw::c_char,
    ptr::null_mut,
};

use log::debug;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AMediaFormat {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[link(name = "mediandk")]
unsafe extern "C" {
    fn AMediaFormat_new() -> *mut AMediaFormat;
    fn AMediaFormat_delete(format: *mut AMediaFormat) -> isize;
    fn AMediaFormat_toString(format: *mut AMediaFormat) -> *const c_char;
    fn AMediaFormat_getInt32(format: *mut AMediaFormat, name: *const c_char, out: *mut i32)
        -> bool;
    fn AMediaFormat_getInt64(format: *mut AMediaFormat, name: *const c_char, out: *mut i64)
        -> bool;
    fn AMediaFormat_getFloat(format: *mut AMediaFormat, name: *const c_char, out: *mut f32)
        -> bool;
    #[cfg(feature = "api28")]
    fn AMediaFormat_getDouble(
        format: *mut AMediaFormat,
        name: *const c_char,
        out: *mut f64,
    ) -> bool;
    #[cfg(feature = "api28")]
    fn AMediaFormat_getRect(
        format: *mut AMediaFormat,
        name: *const c_char,
        left: *mut i32,
        top: *mut i32,
        right: *mut i32,
        bottom: *mut i32,
    ) -> bool;
    fn AMediaFormat_getSize(
        format: *mut AMediaFormat,
        name: *const c_char,
        out: *mut usize,
    ) -> bool;
    fn AMediaFormat_getBuffer(
        format: *mut AMediaFormat,
        name: *const c_char,
        out: *mut *mut c_void,
        size: *mut usize,
    ) -> bool;
    fn AMediaFormat_getString(
        format: *mut AMediaFormat,
        name: *const c_char,
        out: *mut *mut c_char,
    ) -> bool;
    fn AMediaFormat_setInt32(format: *mut AMediaFormat, name: *const c_char, value: i32) -> bool;
    fn AMediaFormat_setInt64(format: *mut AMediaFormat, name: *const c_char, value: i64) -> bool;
    fn AMediaFormat_setFloat(format: *mut AMediaFormat, name: *const c_char, value: f32) -> bool;
    #[cfg(feature = "api28")]
    fn AMediaFormat_setDouble(format: *mut AMediaFormat, name: *const c_char, value: f64) -> bool;
    #[cfg(feature = "api28")]
    fn AMediaFormat_setSize(format: *mut AMediaFormat, name: *const c_char, value: usize) -> bool;
    #[cfg(feature = "api28")]
    fn AMediaFormat_setRect(
        format: *mut AMediaFormat,
        name: *const c_char,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    ) -> bool;
    fn AMediaFormat_setString(
        format: *mut AMediaFormat,
        name: *const c_char,
        value: *const c_char,
    ) -> bool;
    fn AMediaFormat_setBuffer(
        format: *mut AMediaFormat,
        name: *const c_char,
        value: *const c_void,
        size: usize,
    ) -> bool;
    #[cfg(feature = "api29")]
    fn AMediaFormat_clear(format: *mut AMediaFormat);
    #[cfg(feature = "api29")]
    fn AMediaFormat_copy(to: *mut AMediaFormat, from: *mut AMediaFormat) -> isize;
}

#[derive(Debug)]
pub struct MediaFormat {
    pub(crate) inner: *mut AMediaFormat,
}

impl MediaFormat {
    /// Construct a MediaFormat from a raw pointer
    pub fn from_raw(inner: *mut AMediaFormat) -> Self {
        Self { inner }
    }

    pub fn new() -> Option<Self> {
        unsafe {
            let inner = AMediaFormat_new();
            if inner.is_null() {
                return None;
            }
            Some(Self { inner })
        }
    }

    pub fn set_i32(&mut self, name: &str, value: i32) -> bool {
        let name = CString::new(name).unwrap();
        unsafe { AMediaFormat_setInt32(self.inner, name.as_ptr(), value) }
    }

    pub fn get_i32(&self, name: &str) -> Option<i32> {
        unsafe {
            let mut v = 0i32;
            let name = CString::new(name).unwrap();
            AMediaFormat_getInt32(self.inner, name.as_ptr(), &mut v).then_some(v)
        }
    }

    pub fn set_i64(&mut self, name: &str, value: i64) -> bool {
        let name = CString::new(name).unwrap();
        unsafe { AMediaFormat_setInt64(self.inner, name.as_ptr(), value) }
    }

    pub fn get_i64(&self, name: &str) -> Option<i64> {
        unsafe {
            let mut v = 0i64;
            let name = CString::new(name).unwrap();
            AMediaFormat_getInt64(self.inner, name.as_ptr(), &mut v).then_some(v)
        }
    }

    pub fn set_f32(&mut self, name: &str, value: f32) -> bool {
        let name = CString::new(name).unwrap();
        unsafe { AMediaFormat_setFloat(self.inner, name.as_ptr(), value) }
    }

    pub fn get_f32(&self, name: &str) -> Option<f32> {
        unsafe {
            let mut v = 0f32;
            let name = CString::new(name).unwrap();
            AMediaFormat_getFloat(self.inner, name.as_ptr(), &mut v).then_some(v)
        }
    }

    /// Convenience function to check whether the mime type is audio
    pub fn is_audio(&self) -> bool {
        self.get_string("mime").is_some_and(|m| m.contains("audio"))
    }

    pub fn is_video(&self) -> bool {
        self.get_string("mime").is_some_and(|m| m.contains("video"))
    }

    #[cfg(feature = "api28")]
    pub fn set_f64(&mut self, name: &str, value: f64) -> bool {
        let name = CString::new(name).unwrap();
        unsafe { AMediaFormat_setDouble(self.inner, name.as_ptr(), value) }
    }

    #[cfg(feature = "api28")]
    pub fn get_f64(&self, name: &str) -> Option<f64> {
        unsafe {
            let mut v = 0f64;
            let name = CString::new(name).unwrap();
            AMediaFormat_getDouble(self.inner, name.as_ptr(), &mut v).then_some(v)
        }
    }

    pub fn set_string(&mut self, name: &str, value: &str) -> bool {
        let name = CString::new(name).unwrap();
        let value = CString::new(value).unwrap();
        unsafe { AMediaFormat_setString(self.inner, name.as_ptr(), value.as_ptr()) }
    }

    pub fn get_string(&self, name: &str) -> Option<String> {
        unsafe {
            let mut data = null_mut();
            let name = CString::new(name).unwrap();
            if AMediaFormat_getString(self.inner, name.as_ptr(), &mut data) {
                Some(CStr::from_ptr(data).to_string_lossy().into_owned())
            } else {
                None
            }
        }
    }

    /// Clear the entire buffer
    #[cfg(feature = "api29")]
    pub fn clear(&mut self) {
        unsafe {
            AMediaFormat_clear(self.inner);
        }
    }
}

impl fmt::Display for MediaFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            let value = AMediaFormat_toString(self.inner);
            if value.is_null() {
                f.write_str("")
            } else {
                f.write_str(&CStr::from_ptr(value).to_string_lossy())
            }
        }
    }
}

impl Drop for MediaFormat {
    fn drop(&mut self) {
        unsafe {
            AMediaFormat_delete(self.inner);
        }
    }
}

unsafe impl Send for MediaFormat {}
unsafe impl Sync for MediaFormat {}
