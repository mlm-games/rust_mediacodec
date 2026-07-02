use std::{ffi::CString, os::raw::c_char};

use log::debug;

use crate::{AMediaFormat, CodecInputBuffer, MediaFormat, MediaStatus};

#[repr(i32)]
pub enum SeekMode {
    SeekClosestSync = 0,
    SeekNextSync = 1,
    SeekPreviousSync = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct AMediaExtractor {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[link(name = "mediandk")]
unsafe extern "C" {
    fn AMediaExtractor_new() -> *mut AMediaExtractor;
    fn AMediaExtractor_delete(extractor: *mut AMediaExtractor) -> i32;
    fn AMediaExtractor_setDataSourceFd(
        extractor: *mut AMediaExtractor,
        fd: i32,
        offset: i64,
        length: i64,
    ) -> i32;
    fn AMediaExtractor_setDataSource(
        extractor: *mut AMediaExtractor,
        location: *const c_char,
    ) -> i32;
    fn AMediaExtractor_getTrackCount(extractor: *mut AMediaExtractor) -> usize;
    fn AMediaExtractor_getTrackFormat(
        extractor: *mut AMediaExtractor,
        index: usize,
    ) -> *mut AMediaFormat;
    fn AMediaExtractor_selectTrack(extractor: *mut AMediaExtractor, index: usize) -> i32;
    fn AMediaExtractor_unselectTrack(extractor: *mut AMediaExtractor, index: usize) -> i32;
    fn AMediaExtractor_readSampleData(
        extractor: *mut AMediaExtractor,
        buffer: *mut u8,
        capacity: usize,
    ) -> isize;
    fn AMediaExtractor_getSampleFlags(extractor: *mut AMediaExtractor) -> u32;
    fn AMediaExtractor_getSampleTrackIndex(extractor: *mut AMediaExtractor) -> i32;
    fn AMediaExtractor_getSampleTime(extractor: *mut AMediaExtractor) -> i64;
    fn AMediaExtractor_advance(extractor: *mut AMediaExtractor) -> bool;
    fn AMediaExtractor_seekTo(extractor: *mut AMediaExtractor, seek_pos_us: i64, mode: i32);
}

#[derive(Debug)]
pub struct MediaExtractor {
    inner: *mut AMediaExtractor,
    has_next: bool,
}

impl MediaExtractor {
    fn new() -> Self {
        Self {
            inner: unsafe { AMediaExtractor_new() },
            has_next: false,
        }
    }

    /// Creates a MediaExtractor with data source set to a specific URL
    pub fn from_url(path: &str) -> Result<Self, MediaStatus> {
        unsafe {
            let mut me = Self::new();
            let path_cs =
                CString::new(path).map_err(|_| MediaStatus::ErrorInvalidParameter)?;
            MediaStatus::make_result(AMediaExtractor_setDataSource(
                me.inner,
                path_cs.as_ptr(),
            ))?;
            me.has_next = true;
            Ok(me)
        }
    }

    /// Returns the number of tracks found by MediaExtractor
    pub fn track_count(&self) -> usize {
        unsafe { AMediaExtractor_getTrackCount(self.inner) }
    }

    /// Returns the track index of the current packet to be retrieved by MediaExtractor
    pub fn track_index(&self) -> i32 {
        unsafe { AMediaExtractor_getSampleTrackIndex(self.inner) }
    }

    /// Returns the MediaFormat containing the parameters for this track index.
    ///
    /// The format can be used to create and initialize MediaCodec
    pub fn track_format(&self, index: usize) -> Option<MediaFormat> {
        unsafe {
            if self.track_count() <= index {
                debug!("Invalid track index {index}");
                return None;
            }
            let fmt = AMediaExtractor_getTrackFormat(self.inner, index);
            Some(MediaFormat::from_raw(fmt))
        }
    }

    /// Select this track to be demuxed by MediaExtractor
    pub fn select_track(&mut self, index: usize) -> Result<(), MediaStatus> {
        unsafe { MediaStatus::make_result(AMediaExtractor_selectTrack(self.inner, index)) }
    }

    /// Unselect this track to be demuxed by MediaExtractor
    pub fn unselect_track(&mut self, index: usize) -> Result<(), MediaStatus> {
        unsafe { MediaStatus::make_result(AMediaExtractor_unselectTrack(self.inner, index)) }
    }

    /// Returns the sample flags for the current packet to be returned
    pub fn sample_flags(&self) -> u32 {
        unsafe { AMediaExtractor_getSampleFlags(self.inner) }
    }

    /// Returns the time for the current packet to be returned
    pub fn sample_time(&self) -> i64 {
        unsafe { AMediaExtractor_getSampleTime(self.inner) }
    }

    /// Read a packet into `buffer` and advance the extractor
    /// Returns `Ok(true)` if data was read, `Ok(false)` if no more data, or `Err` on error
    pub fn read_next(&mut self, buffer: &mut CodecInputBuffer) -> Result<bool, MediaStatus> {
        unsafe {
            if !self.has_next {
                return Ok(false);
            }
            let count =
                AMediaExtractor_readSampleData(self.inner, buffer.raw_buffer(), buffer.raw_size());
            if count > 0 {
                buffer.set_write_size(count as usize);

                let sample_time = self.sample_time();
                if sample_time >= 0 {
                    buffer.set_time(sample_time as u64);
                }

                // Extractor SAMPLE_FLAG_SYNC (1) = Codec BUFFER_FLAG_KEY_FRAME (1).
                // Extractor SAMPLE_FLAG_ENCRYPTED (2) MUST NOT be forwarded as BUFFER_FLAG_CODEC_CONFIG (2).
                // Only forward known codec-relevant flags.
                let extractor_flags = self.sample_flags();
                let mut codec_flags = extractor_flags & 0xD; // Keep only flags 1 (KEY_FRAME), 4 (EOS...), 8 (PARTIAL_FRAME)
                codec_flags &= !2; // Explicitly strip encrypted flag (not a codec buffer flag)
                buffer.set_flags(codec_flags);
            } else if count < 0 {
                buffer.cancel();
                return Err(MediaStatus::from_i32(count as i32));
            }
            self.has_next = AMediaExtractor_advance(self.inner);
            if count > 0 {
                Ok(true)
            } else {
                buffer.cancel();
                Ok(false)
            }
        }
    }

    /// Returns whether MediaExtractor still has packets to read
    pub fn has_next(&self) -> bool {
        self.has_next
    }

    pub fn seek_to(&mut self, pos_us: i64, mode: SeekMode) {
        unsafe {
            AMediaExtractor_seekTo(self.inner, pos_us, mode as i32);
        }
        self.has_next = true;
    }
}

impl Drop for MediaExtractor {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                AMediaExtractor_delete(self.inner);
            }
        }
    }
}

unsafe impl Send for MediaExtractor {}
