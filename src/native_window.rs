use std::{ffi::c_void, ops::BitOr, ptr::null_mut};

use jni::objects::JObject;

#[repr(C)]
#[derive(Debug)]
pub struct ANativeWindow {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[derive(Debug, Clone, Copy)]
pub enum NativeWindowFormat {
    Rgba8 = 1,
    Rgb8 = 2,
    Rgb565 = 4,
    Yuv420 = 0x23,
    Other,
}

impl NativeWindowFormat {
    fn values() -> &'static [Self] {
        use NativeWindowFormat::*;
        &[Rgba8, Rgb8, Rgb565, Yuv420]
    }
}

impl From<isize> for NativeWindowFormat {
    fn from(value: isize) -> Self {
        for &item in Self::values() {
            if item as isize == value {
                return item;
            }
        }
        Self::Other
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NativeWindowTransform {
    Identity = 0x00,
    MirrorHorizontal = 0x01,
    MirrorVertical = 0x02,
    Rotate90 = 0x04,
    Rotate180 = NativeWindowTransform::MirrorHorizontal as isize
        | NativeWindowTransform::MirrorVertical as isize,
    Rotate270 =
        NativeWindowTransform::Rotate180 as isize | NativeWindowTransform::Rotate90 as isize,
}

impl BitOr for NativeWindowTransform {
    type Output = isize;
    fn bitor(self, rhs: Self) -> Self::Output {
        self as isize | rhs as isize
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct NativeWindowBuffer {
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub format: i32,
    pub bits: *mut c_void,
    reserved: [u32; 6],
    window: *mut ANativeWindow,
}

impl NativeWindowBuffer {
    fn new(window: *mut ANativeWindow) -> Self {
        unsafe { ANativeWindow_acquire(window) };
        Self {
            width: 0,
            height: 0,
            stride: 0,
            format: 0,
            reserved: [0; 6],
            bits: null_mut(),
            window,
        }
    }
}

impl Drop for NativeWindowBuffer {
    fn drop(&mut self) {
        if !self.window.is_null() {
            unsafe {
                ANativeWindow_unlockAndPost(self.window);
            }
            NativeWindow::from_raw(self.window);
        }
    }
}

#[repr(C)]
pub struct ARect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[link(name = "android")]
unsafe extern "C" {
    fn ANativeWindow_fromSurface(
        env: *mut jni::sys::JNIEnv,
        surface: jni::sys::jobject,
    ) -> *mut ANativeWindow;
    #[cfg(feature = "api26")]
    fn ANativeWindow_toSurface(
        env: *mut jni::sys::JNIEnv,
        window: *mut ANativeWindow,
    ) -> jni::sys::jobject;
    fn ANativeWindow_acquire(window: *mut ANativeWindow);
    fn ANativeWindow_release(window: *mut ANativeWindow);
    fn ANativeWindow_getWidth(window: *mut ANativeWindow) -> i32;
    fn ANativeWindow_getHeight(window: *mut ANativeWindow) -> i32;
    fn ANativeWindow_getFormat(window: *mut ANativeWindow) -> i32;
    fn ANativeWindow_setBuffersGeometry(
        window: *mut ANativeWindow,
        width: i32,
        height: i32,
        format: i32,
    ) -> i32;
    fn ANativeWindow_lock(
        window: *mut ANativeWindow,
        buffer: *mut NativeWindowBuffer,
        rect: *mut ARect,
    ) -> i32;
    fn ANativeWindow_unlockAndPost(window: *mut ANativeWindow) -> i32;
}

#[derive(Debug)]
pub struct NativeWindow {
    pub(crate) inner: *mut ANativeWindow,
}

impl NativeWindow {
    pub fn from_raw(inner: *mut ANativeWindow) -> Self {
        Self { inner }
    }

    pub fn from_surface(surface: JObject) -> Self {
        let ctx = ndk_context::android_context();
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) };
        let raw_env = vm
            .attach_current_thread(|env| -> Result<_, jni::errors::Error> { Ok(env.get_raw()) })
            .expect("failed to attach thread");
        unsafe { Self::from_raw(ANativeWindow_fromSurface(raw_env, surface.as_raw())) }
    }

    #[cfg(feature = "api26")]
    pub fn to_surface<'local>(&self, env: &mut JNIEnv<'local>) -> JObject<'local> {
        unsafe { JObject::from_raw(ANativeWindow_toSurface(env.get_raw(), self.inner)) }
    }

    pub fn width(&self) -> i32 {
        unsafe { ANativeWindow_getWidth(self.inner) }
    }

    pub fn height(&self) -> i32 {
        unsafe { ANativeWindow_getHeight(self.inner) }
    }

    pub fn format(&self) -> NativeWindowFormat {
        unsafe { NativeWindowFormat::from(ANativeWindow_getFormat(self.inner) as isize) }
    }

    pub fn set_geometry(&mut self, width: i32, height: i32, format: NativeWindowFormat) {
        unsafe {
            ANativeWindow_setBuffersGeometry(self.inner, width, height, format as i32);
        }
    }

    pub fn lock(&mut self, bounds: &mut ARect) -> Option<NativeWindowBuffer> {
        unsafe {
            let mut buffer = NativeWindowBuffer::new(self.inner);
            if ANativeWindow_lock(self.inner, &mut buffer, bounds) == 0 {
                return Some(buffer);
            }
        }
        None
    }
}

impl Clone for NativeWindow {
    fn clone(&self) -> Self {
        unsafe {
            ANativeWindow_acquire(self.inner);
        }
        Self { inner: self.inner }
    }
}

impl Drop for NativeWindow {
    fn drop(&mut self) {
        unsafe {
            ANativeWindow_release(self.inner);
        }
    }
}

unsafe impl Send for NativeWindow {}
unsafe impl Sync for NativeWindow {}
