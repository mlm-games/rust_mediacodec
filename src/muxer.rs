use log::warn;

use crate::{AMediaFormat, BufferInfo, MediaFormat, MediaStatus};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AMediaMuxer {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug)]
pub enum OutputFormat {
    Mpeg4 = 0,
    Webm = 1,
    ThreeGpp = 2,
}

// FFI FUNCTIONS

#[link(name = "mediandk")]
unsafe extern "C" {
    /// Since: API 21
    fn AMediaMuxer_new(fd: i32, format: OutputFormat) -> *mut AMediaMuxer;

    /// Since: API 21
    fn AMediaMuxer_delete(muxer: *mut AMediaMuxer) -> i32;

    /// Since: API 21
    fn AMediaMuxer_setLocation(
        muxer: *mut AMediaMuxer,
        latitude: f32,
        longitude: f32,
    ) -> i32;

    /// Since: API 21
    fn AMediaMuxer_setOrientationHint(muxer: *mut AMediaMuxer, degrees: i32) -> i32;

    /// Since: API 21
    fn AMediaMuxer_addTrack(muxer: *mut AMediaMuxer, format: *const AMediaFormat) -> isize;

    /// Since: API 21
    fn AMediaMuxer_start(muxer: *mut AMediaMuxer) -> i32;

    /// Since: API 21
    fn AMediaMuxer_stop(muxer: *mut AMediaMuxer) -> i32;

    /// Since: API 21
    fn AMediaMuxer_writeSampleData(
        muxer: *mut AMediaMuxer,
        track_index: usize,
        data: *const u8,
        info: *const BufferInfo,
    ) -> i32;
}

// FFI FUNCTIONS END

#[derive(Debug, Eq, PartialEq)]
enum MuxerState {
    Uninitialized,
    Started,
}

/// The Type-Safe wrapper for `AMediaMuxer`.
///
/// Ensures memory safety and frees resources when it's supposed to.
///
/// Also comes with some extra super powers that ensures user supplies correct information before handing them to `AMediaMuxer`
#[derive(Debug)]
pub struct MediaMuxer {
    inner: *mut AMediaMuxer,
    latitude: f32,
    longitude: f32,
    location_set: bool,
    orientation_hint: i32,
    track_formats: Vec<MediaFormat>,
    state: MuxerState,
}

impl MediaMuxer {
    /// Creates a new MediaMuxer instance
    ///
    /// `fd` is the file descriptor to write data to
    ///
    /// `output_format` is the container format for the output
    pub fn new(fd: i32, output_format: OutputFormat) -> Result<Self, MediaStatus> {
        let value = unsafe { AMediaMuxer_new(fd, output_format) };

        if value.is_null() {
            return Err(MediaStatus::ErrorUnknown);
        }

        Ok(Self {
            inner: value,
            latitude: 0f32,
            longitude: 0f32,
            location_set: false,
            orientation_hint: 0,
            track_formats: vec![],
            state: MuxerState::Uninitialized,
        })
    }

    /// Set and store the geodata (latitude and longitude) in the output file.
    /// This method should be called before calling `start`. The geodata is stored in udata box if the output format is Mpeg4, and is ignored for other output formats.
    ///
    /// The geodata is stored according to ISO-6709 standard.
    ///
    /// Both values are specified in degrees.
    ///
    /// Latitude must be in the range (-90, 90)
    ///
    /// Longitude must be in the range (-180, 180)
    pub fn set_location(&mut self, latitude: f32, longitude: f32) -> Result<&mut Self, MediaStatus> {
        if !latitude.is_finite()
            || !longitude.is_finite()
            || !(-90.0..=90.0).contains(&latitude)
            || !(-180.0..=180.0).contains(&longitude)
        {
            return Err(MediaStatus::ErrorInvalidParameter);
        }
        self.latitude = latitude;
        self.longitude = longitude;
        self.location_set = true;
        Ok(self)
    }

    /// Sets the orientation hint for output video playback.
    ///
    /// This method should be called before calling start. Calling this method will not rotate the video frame when muxer is generating the file, but add a composition matrix containing the rotation angle in the output video if the output format is Mpeg4, so that a video player can choose the proper orientation for playback.
    /// Note that some video players may choose to ignore the composition matrix during playback.
    ///
    /// The angle is specified in degrees, clockwise.
    ///
    /// The supported angles are: 0, 90, 180 and 270 degrees.
    pub fn set_orientation_hint(&mut self, degrees: i32) -> &mut Self {
        match degrees {
            0 | 90 | 180 | 270 => self.orientation_hint = degrees,
            hint => warn!("Unsupported orientation hint passed to MediaMuxer: {hint}"),
        }

        self
    }

    /// Adds a track with the specified format.
    ///
    /// Returns the index of the new track or a `MediaStatus` in case of failure.
    pub fn add_track(&mut self, format: MediaFormat) -> Result<i32, MediaStatus> {
        let result = unsafe { AMediaMuxer_addTrack(self.inner, format.inner.ptr) };

        if result < 0 {
            return Err(MediaStatus::from_i32(result as i32));
        }
        let track_index = result as i32;

        // Keep the format, the user might need it
        self.track_formats.push(format);

        Ok(track_index)
    }

    /// Returns the number of tracks added to the muxer
    pub fn track_count(&self) -> usize {
        self.track_formats.len()
    }

    /// Returns the track format for a specific track
    pub fn format(&self, index: usize) -> Option<&MediaFormat> {
        if index >= self.track_formats.len() {
            return None;
        }

        Some(&self.track_formats[index])
    }

    /// Start the muxer. Should be called only after tracks have been added.
    pub fn start(&mut self) -> Result<(), MediaStatus> {
        if let MuxerState::Started = self.state {
            return Ok(());
        }

        // Make sure they've added at least one track
        if self.track_formats.is_empty() {
            return Err(MediaStatus::ErrorInvalidOperation);
        }

        unsafe {
            if self.location_set {
                MediaStatus::make_result(AMediaMuxer_setLocation(
                    self.inner,
                    self.latitude,
                    self.longitude,
                ))?;
            }
            if self.orientation_hint != 0 {
                MediaStatus::make_result(AMediaMuxer_setOrientationHint(
                    self.inner,
                    self.orientation_hint,
                ))?;
            }

            // Start the muxer
            MediaStatus::make_result(AMediaMuxer_start(self.inner))?;

            self.state = MuxerState::Started;
        }

        Ok(())
    }

    /// Stops the muxer.
    ///
    /// Once the muxer stops, it cannot be restarted, and therefore this function takes ownership
    /// of the muxer instance
    pub fn stop(self) -> Result<(), MediaStatus> {
        if let MuxerState::Uninitialized = self.state {
            return Err(MediaStatus::ErrorInvalidOperation);
        }

        unsafe { MediaStatus::make_result(AMediaMuxer_stop(self.inner)) }
    }

    /// Writes an encoded sample into the muxer.
    ///
    /// The application needs to make sure that the samples are written into the right tracks.
    ///
    /// Also, it needs to make sure the samples for each track are written in chronological order (e.g. in the order they are provided by the encoder)
    pub fn write_sample_data(
        &mut self,
        track_index: usize,
        data: &[u8],
        buffer_info: &BufferInfo,
    ) -> Result<(), MediaStatus> {
        if let MuxerState::Uninitialized = self.state {
            return Err(MediaStatus::ErrorInvalidOperation);
        }
        if track_index >= self.track_formats.len() {
            return Err(MediaStatus::ErrorInvalidParameter);
        }
        if buffer_info.offset < 0 || buffer_info.size < 0 {
            return Err(MediaStatus::ErrorInvalidParameter);
        }
        let end = (buffer_info.offset as usize)
            .checked_add(buffer_info.size as usize)
            .ok_or(MediaStatus::ErrorInvalidParameter)?;
        if end > data.len() {
            return Err(MediaStatus::ErrorInvalidParameter);
        }
        unsafe {
            MediaStatus::make_result(AMediaMuxer_writeSampleData(
                self.inner,
                track_index,
                data.as_ptr(),
                buffer_info,
            ))
        }
    }
}

impl Drop for MediaMuxer {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                AMediaMuxer_delete(self.inner);
            }
        }
    }
}
