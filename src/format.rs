use std::{
    ffi::{CStr, CString, c_void},
    fmt,
    os::raw::c_char,
    ptr::null_mut,
    rc::Rc,
};

use crate::MediaStatus;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AMediaFormat {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[link(name = "mediandk")]
unsafe extern "C" {
    fn AMediaFormat_new() -> *mut AMediaFormat;
    fn AMediaFormat_delete(format: *mut AMediaFormat) -> i32;
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
    fn AMediaFormat_copy(to: *mut AMediaFormat, from: *mut AMediaFormat) -> i32;
}

#[derive(Debug)]
pub(crate) struct MediaFormatInner {
    pub(crate) ptr: *mut AMediaFormat,
}

impl Drop for MediaFormatInner {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                AMediaFormat_delete(self.ptr);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct MediaFormat {
    pub(crate) inner: Rc<MediaFormatInner>,
}

fn cstr(s: &str) -> CString {
    CString::new(s).expect("format key/value must not contain null bytes")
}

impl MediaFormat {
    /// Construct a MediaFormat from a raw pointer
    ///
    /// Safety:
    /// `inner` must be a valid, non-null pointer to an `AMediaFormat` that this
    /// `MediaFormat` will own. The caller must ensure the pointer is valid for the
    /// lifetime of the returned `MediaFormat`.
    pub unsafe fn from_raw(ptr: *mut AMediaFormat) -> Self {
        Self {
            inner: Rc::new(MediaFormatInner { ptr }),
        }
    }

    pub fn new() -> Result<Self, MediaStatus> {
        unsafe {
            let ptr = AMediaFormat_new();
            if ptr.is_null() {
                return Err(MediaStatus::ErrorUnknown);
            }
            Ok(Self {
                inner: Rc::new(MediaFormatInner { ptr }),
            })
        }
    }

    #[must_use]
    pub fn set_i32(&mut self, name: &str, value: i32) -> bool {
        let name = cstr(name);
        unsafe { AMediaFormat_setInt32(self.inner.ptr, name.as_ptr(), value) }
    }

    #[must_use]
    pub fn get_i32(&self, name: &str) -> Option<i32> {
        unsafe {
            let mut v = 0i32;
            let name = cstr(name);
            AMediaFormat_getInt32(self.inner.ptr, name.as_ptr(), &mut v).then_some(v)
        }
    }

    #[must_use]
    pub fn set_i64(&mut self, name: &str, value: i64) -> bool {
        let name = cstr(name);
        unsafe { AMediaFormat_setInt64(self.inner.ptr, name.as_ptr(), value) }
    }

    #[must_use]
    pub fn get_i64(&self, name: &str) -> Option<i64> {
        unsafe {
            let mut v = 0i64;
            let name = cstr(name);
            AMediaFormat_getInt64(self.inner.ptr, name.as_ptr(), &mut v).then_some(v)
        }
    }

    #[must_use]
    pub fn set_f32(&mut self, name: &str, value: f32) -> bool {
        let name = cstr(name);
        unsafe { AMediaFormat_setFloat(self.inner.ptr, name.as_ptr(), value) }
    }

    #[must_use]
    pub fn get_f32(&self, name: &str) -> Option<f32> {
        unsafe {
            let mut v = 0f32;
            let name = cstr(name);
            AMediaFormat_getFloat(self.inner.ptr, name.as_ptr(), &mut v).then_some(v)
        }
    }

    #[must_use]
    pub fn is_audio(&self) -> bool {
        self.get_string("mime").is_some_and(|m| m.starts_with("audio/"))
    }

    #[must_use]
    pub fn is_video(&self) -> bool {
        self.get_string("mime").is_some_and(|m| m.starts_with("video/"))
    }

    #[cfg(feature = "api28")]
    #[must_use]
    pub fn set_f64(&mut self, name: &str, value: f64) -> bool {
        let name = cstr(name);
        unsafe { AMediaFormat_setDouble(self.inner.ptr, name.as_ptr(), value) }
    }

    #[cfg(feature = "api28")]
    #[must_use]
    pub fn get_f64(&self, name: &str) -> Option<f64> {
        unsafe {
            let mut v = 0f64;
            let name = cstr(name);
            AMediaFormat_getDouble(self.inner.ptr, name.as_ptr(), &mut v).then_some(v)
        }
    }

    #[must_use]
    pub fn set_string(&mut self, name: &str, value: &str) -> bool {
        let name = cstr(name);
        let value = cstr(value);
        unsafe { AMediaFormat_setString(self.inner.ptr, name.as_ptr(), value.as_ptr()) }
    }

    #[must_use]
    pub fn set_buffer(&mut self, name: &str, value: &[u8]) -> bool {
        let name = cstr(name);
        unsafe {
            AMediaFormat_setBuffer(
                self.inner.ptr,
                name.as_ptr(),
                value.as_ptr() as *const c_void,
                value.len(),
            )
        }
    }

    pub fn get_string(&self, name: &str) -> Option<String> {
        unsafe {
            let mut data = null_mut();
            let name = cstr(name);
            if AMediaFormat_getString(self.inner.ptr, name.as_ptr(), &mut data) {
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
            AMediaFormat_clear(self.inner.ptr);
        }
    }
}

impl fmt::Display for MediaFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            let value = AMediaFormat_toString(self.inner.ptr);
            if value.is_null() {
                f.write_str("")
            } else {
                f.write_str(&CStr::from_ptr(value).to_string_lossy())
            }
        }
    }
}


