use log::{debug, warn};

use crate::{
    AMediaCrypto, AMediaFormat, ANativeWindow, AudioFrame, ENCODING_PCM_16BIT, ENCODING_PCM_FLOAT,
    Frame, MediaFormat, MediaStatus, NativeWindow, SampleFormat, VideoFrame,
};
use std::{
    borrow::Cow,
    ffi::{CString, c_void},
    marker::PhantomData,
    os::raw::c_char,
    ptr::null_mut,
    slice,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct BufferInfo {
    pub offset: i32,
    pub size: i32,
    pub presentation_time_us: i64,
    pub flags: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DequeueInputError {
    TryAgainLater,
    CodecError(MediaStatus),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DequeueOutputError {
    TryAgainLater,
    OutputFormatChanged,
    OutputBuffersChanged,
    CodecError(MediaStatus),
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AMediaCodecCryptoInfo {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct AMediaCodec {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[derive(Clone, Copy, Debug)]
pub enum BufferFlag {
    KeyFrame = 1,
    CodecConfig = 2,
    EndOfStream = 4,
    PartialFrame = 8,
}

impl TryFrom<i32> for BufferFlag {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::KeyFrame),
            2 => Ok(Self::CodecConfig),
            4 => Ok(Self::EndOfStream),
            8 => Ok(Self::PartialFrame),
            _ => Err(String::from("Not Found")),
        }
    }
}

impl TryFrom<BufferFlag> for i32 {
    type Error = String;

    fn try_from(value: BufferFlag) -> Result<Self, Self::Error> {
        Ok(value as i32)
    }
}

impl BufferFlag {
    pub fn is_contained_in(&self, flag: i32) -> bool {
        flag & (*self as i32) > 0
    }

    pub fn add_to_flag(&self, flag: &mut i32) {
        *flag |= *self as i32;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum InfoFlag {
    OutputBuffersChanged = -3,
    OutputFormatChanged = -2,
    TryAgainLater = -1,
}

impl TryFrom<i32> for InfoFlag {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            -3 => Ok(Self::OutputBuffersChanged),
            -2 => Ok(Self::OutputFormatChanged),
            -1 => Ok(Self::TryAgainLater),
            _ => Err(String::from("Not Found")),
        }
    }
}

impl TryFrom<InfoFlag> for i32 {
    type Error = String;

    fn try_from(value: InfoFlag) -> Result<Self, Self::Error> {
        Ok(value as i32)
    }
}



#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CryptoInfoMode {
    Clear = 0,
    AesCtr = 1,
    AesWv = 2,
    AesCbc = 3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CryptoInfoPattern {
    pub encrypt_blocks: i32,
    pub skip_blocks: i32,
}

type _AMediaCodecOnAsyncInputAvailable = extern "C" fn(
    // Codec
    *const AMediaCodec,
    // Userdata
    userdata: *const c_void,
    // Index
    index: i32,
);

type _AMediaCodecOnAsyncOutputAvailable = extern "C" fn(
    // Codec
    *const AMediaCodec,
    // Userdata
    *const c_void,
    // Index
    i32,
    // Buffer info
    *const BufferInfo,
);

type _AMediaCodecOnAsyncFormatChanged = extern "C" fn(
    // Codec
    *const AMediaCodec,
    // Userdata
    *const c_void,
    // Format
    *const AMediaFormat,
);

type _AMediaCodecOnAsyncError = extern "C" fn(
    // Codec
    *const AMediaCodec,
    // Userdata
    *const c_void,
    // Error
    i32,
    // Action code
    i32,
    // Details
    *const c_char,
);

#[repr(C)]
struct _AMediaCodecOnAsyncNotifyCallback {
    on_async_input_available: _AMediaCodecOnAsyncInputAvailable,
    on_async_output_available: _AMediaCodecOnAsyncOutputAvailable,
    on_async_format_changed: _AMediaCodecOnAsyncFormatChanged,
    on_async_error: _AMediaCodecOnAsyncError,
}

// FFI FUNCTIONS BEGIN

#[link(name = "mediandk")]
unsafe extern "C" {
    /// Create codec by name. Use this if you know the exact codec you want to use.
    /// When configuring, you will need to specify whether to use the codec as an encoder or decoder.
    /// <hr />
    /// Since: API 21
    fn AMediaCodec_createCodecByName(name: *const c_char) -> *mut AMediaCodec;

    /// Create codec by mime type. Most applications will use this, specifying a mime type obtained from media extractor.
    /// <hr />
    /// Since: API 21
    fn AMediaCodec_createDecoderByType(mime_type: *const c_char) -> *mut AMediaCodec;

    /// Create encoder by mime type.
    /// <hr />
    /// Since: API 21
    fn AMediaCodec_createEncoderByType(mime_type: *const c_char) -> *mut AMediaCodec;

    /// Delete the codec and free its resources
    /// <hr />
    /// Since: API 21
    fn AMediaCodec_delete(codec: *mut AMediaCodec) -> i32;

    /// Configure the codec. For decoding, you would typically get the format from an extractor
    /// <hr />
    /// Since: API 21
    fn AMediaCodec_configure(
        codec: *mut AMediaCodec,
        format: *const AMediaFormat,
        surface: *mut ANativeWindow,
        crypto: *mut AMediaCrypto,
        flags: u32,
    ) -> i32;

    /// Start the codec. A codec must be configured before it can be started, and must be started before buffers can be sent to it.
    /// <hr />
    /// Since: API 21
    fn AMediaCodec_start(codec: *mut AMediaCodec) -> i32;

    /// Stop the codec.
    /// <hr />
    /// Since: API 21
    fn AMediaCodec_stop(codec: *mut AMediaCodec) -> i32;

    /// Flush the codec's input and output. All indices previously returned from calls to `AMediaCodec_dequeueInputBuffer` and `AMediaCodec_dequeueOutputBuffer` become invalid.
    /// <hr />
    /// Since: API 21
    fn AMediaCodec_flush(codec: *mut AMediaCodec) -> i32;

    /// Get an input buffer. The specified buffer index must have been previously obtained from dequeueInputBuffer, and not yet queued.
    /// <hr />
    /// Since: API 21
    fn AMediaCodec_getInputBuffer(
        codec: *mut AMediaCodec,
        idx: usize,
        out_size: *mut usize,
    ) -> *mut u8;

    /// Get an output buffer. The specified buffer index must have been previously obtained from `dequeueOutputBuffer`, and not yet queued.
    /// <hr />
    /// Since: API 21
    fn AMediaCodec_getOutputBuffer(
        codec: *mut AMediaCodec,
        idx: usize,
        out_size: *mut usize,
    ) -> *mut u8;

    /// Get the index of the next available input buffer. An app will typically use this with `getInputBuffer` to get a pointer to the buffer, then copy the data to be encoded or decoded into the buffer before passing it to the codec.
    /// <hr />
    /// Since: API 21
    fn AMediaCodec_dequeueInputBuffer(codec: *mut AMediaCodec, timeout_us: i64) -> isize;

    /// Send the specified buffer to the codec for processing
    /// <hr />
    /// Since: API 21
    fn AMediaCodec_queueInputBuffer(
        codec: *mut AMediaCodec,
        idx: usize,
        offset: usize,
        size: usize,
        time: u64,
        flags: u32,
    ) -> i32;

    /// Send the specified buffer to the codec for processing
    /// <hr />
    /// Since: API 21
    fn AMediaCodec_queueSecureInputBuffer(
        codec: *mut AMediaCodec,
        idx: usize,
        offset: usize,
        info: *mut AMediaCodecCryptoInfo,
        time: u64,
        flags: u32,
    ) -> i32;

    /// Get the index of the next available buffer of processed data
    /// <hr />
    // Since: API 21
    fn AMediaCodec_dequeueOutputBuffer(
        codec: *mut AMediaCodec,
        info: *mut BufferInfo,
        timeout_us: i64,
    ) -> isize;

    /// Returns the format of the codec's output.
    /// The caller must free the returned format
    /// <hr />
    // Since: API 21
    fn AMediaCodec_getOutputFormat(codec: *mut AMediaCodec) -> *mut AMediaFormat;

    /// If you are done with a buffer, use this call to return the buffer to the codec. If you previously specified a surface when configuring this video decoder, you can optionally render the buffer.
    /// <hr />
    // Since: API 21
    fn AMediaCodec_releaseOutputBuffer(
        codec: *mut AMediaCodec,
        index: usize,
        render: bool,
    ) -> i32;

    /// Dynamically sets the output surface of a codec.
    /// This can only be used if the codec was configured with an output surface. The new output surface should have a compatible usage type to the original output surface. E.g. Codecs may not support switching from a SurfaceTexture (GPU readable) output to ImageReader (software readable) output.
    /// <hr />
    // Since: API 21
    fn AMediaCodec_setOutputSurface(
        codec: *mut AMediaCodec,
        surface: *mut ANativeWindow,
    ) -> i32;

    /// If you are done with a buffer, use this call to update its surface timestamp and return it to the codec to render it to the output surface. If you have not specified an output surface when configuring this video codec, this call will simply return the buffer to the codec.
    /// <hr />
    // Since: API 21
    fn AMediaCodec_releaseOutputBufferAtTime(
        codec: *mut AMediaCodec,
        idx: usize,
        timestamp_ns: i64,
    );

    /// Creates a surface that can be used as input to encoder, in place of input buffers.
    ///
    /// This can only be called after the codec has been configured via `AMediaCodec_configure` and before `AMediaCodec_start` has been called.
    ///
    /// The application is responsible for releasing the surface by calling `ANativeWindow_release` when done.
    ///
    /// <hr />
    /// Since: API 26
    #[cfg(feature = "api26")]
    fn AMediaCodec_createInputSurface(
        codec: *mut AMediaCodec,
        surface: *mut *mut ANativeWindow,
    ) -> i32;

    /// Creates a persistent surface that can be used as the input to encoder.
    ///
    /// Persistent surface can be reused by MediaCodec instances and can be set on a new instance via `AMediaCodec_setInputSurface`. A persistent surface can be connected to at most one instance of MediaCodec at any point in time.
    ///
    /// The application is responsible for releasing the surface by calling `ANativeWindow_release` when done.
    ///
    ///<hr />
    /// Since: API 26
    #[cfg(feature = "api26")]
    fn AMediaCodec_createPersistentInputSurface(surface: *mut *mut ANativeWindow) -> i32;

    /// Set a persistent surface that can be used as input to encoder, in place of input buffers
    ///
    /// The surface provided **must** be a persistent surface created via `AMediaCodec_createPersistentInputSurface`.
    /// This can only be called after the codec has been configured by calling `AMediaCodec_configure` and before `AMediaCodec_start` has been called.
    ///
    /// <hr />
    /// Since: API 26
    #[cfg(feature = "api26")]
    fn AMediaCodec_setInputSurface(
        codec: *mut AMediaCodec,
        surface: *mut ANativeWindow,
    ) -> i32;

    /// Signal additional parameters to the codec instance.
    ///
    /// Parameters can be communicated only when the codec is running, i.e. after `AMediaCodec_start` has been called.
    ///
    /// **NOTE:** Some of these parameter changes may silently fail to apply.
    ///
    /// <hr />
    /// Since: API 26
    #[cfg(feature = "api26")]
    fn AMediaCodec_setParameters(
        codec: *mut AMediaCodec,
        format: *const AMediaFormat,
    ) -> i32;

    /// Signals end-of-stream on input. Equivalent to submitting an empty buffer with `AMEDIACODEC_BUFFER_FLAG_END_OF_STREAM` set.
    ///
    /// Returns `AMEDIA_ERROR_INVALID_OPERATION` when used with an encoder not in executing state or not receiving input from a Surface created from `AMediaCodec_createInputSurface` or `AMediaCodec_createPersistentInputSurface`.
    ///
    /// Returns the previous codec error if one exists.
    /// Return AMEDIA_OK when completed successfully.
    ///
    /// <hr />
    /// Since: API 26
    #[cfg(feature = "api26")]
    fn AMediaCodec_signalEndOfInputStream(codec: *mut AMediaCodec) -> i32;

    /// Get format of the buffer. The specified buffer index must have been previously obtained from `dequeueOutputBuffer`.
    /// The caller must free the returned format.
    /// <br />
    /// Since: API 28
    #[cfg(feature = "api28")]
    fn AMediaCodec_getBufferFormat(codec: *mut AMediaCodec, index: usize) -> *mut AMediaFormat;

    /// Get the component name. If the codec was created by `createDecoderByType` or `createEncoderByType`, what component is chosen is not known beforehand. Caller shall call `AMediaCodec_releaseName` to free the returned pointer.
    /// <hr />
    /// Since: API 28
    #[cfg(feature = "api28")]
    fn AMediaCodec_getName(codec: *mut AMediaCodec, out_name: *mut *mut c_char) -> i32;

    /// Free the memory pointed to by name which is returned by AMediaCodec_getName.
    /// <hr />
    /// Since: API 28.
    #[cfg(feature = "api28")]
    fn AMediaCodec_releaseName(codec: *mut AMediaCodec, name: *mut c_char);

    /// Set an asynchronous callback for actionable AMediaCodec events.
    /// When asynchronous callback is enabled, the client should not call `AMediaCodec_getInputBuffer`, `AMediaCodec_getOutputBuffer`, `AMediaCodec_dequeueInputBuffer` or `AMediaCodec_dequeueOutputBuffer`.
    ///
    /// Also, `AMediaCodec_flush` behaves differently in asynchronous mode.
    /// After calling AMediaCodec_flush, you must call AMediaCodec_start to "resume" receiving input buffers, even if an input surface was created.
    ///
    /// The specified userdata is the pointer used when those callback functions are called.
    ///
    /// All callbacks are fired on one NDK internal thread.
    /// `AMediaCodec_setAsyncNotifyCallback` should not be called on the callback thread.
    /// No heavy duty task should be performed on the callback thread.
    ///
    /// <hr />
    /// Since: API 28
    #[cfg(feature = "api28")]
    fn AMediaCodec_setAsyncNotifyCallback(
        codec: *mut AMediaCodec,
        callback: _AMediaCodecOnAsyncNotifyCallback,
        userdata: *mut c_void,
    );

    /// Release the crypto if applicable.
    /// <hr />
    /// Since: API 28.
    #[cfg(feature = "api28")]
    fn AMediaCodec_releaseCrypto(codec: *mut AMediaCodec) -> i32;

    /// Call this after `AMediaCodec_configure` returns successfully to get the input format accepted by the codec. Do this to determine what optional configuration parameters were supported by the codec.
    ///
    /// The caller must free the returned format
    /// <hr />
    /// Since: API 28
    #[cfg(feature = "api28")]
    fn AMediaCodec_getInputFormat(codec: *mut AMediaCodec) -> *mut AMediaFormat;

    /// Returns true if the codec cannot proceed further, but can be recovered by stopping, configuring and starting again.
    ///
    /// <hr />
    /// Since: API 28.
    #[cfg(feature = "api28")]
    fn AMediaCodecActionCode_isRecoverable(action_code: i32) -> bool;

    /// Returns true if the codec error is a transient issue perhaps due to resource constraints, and that the method (or encoding/decoding) may be retried at a later time.
    ///
    /// <hr />
    /// Since: API 28.
    #[cfg(feature = "api28")]
    fn AMediaCodecActionCode_isTransient(action_code: i32) -> bool;

    /// Since: API 21
    fn AMediaCodecCryptoInfo_new(
        num_subsamples: i32,
        key: &[u8; 16],
        iv: &[u8; 16],
        mode: CryptoInfoMode,
        clearbytes: *mut usize,
        encrypted_bytes: *mut usize,
    ) -> *mut AMediaCodecCryptoInfo;

    /// Since: API 21
    fn AMediaCodecCryptoInfo_delete(info: *mut AMediaCodecCryptoInfo) -> i32;

    /// Since: API 21
    fn AMediaCodecCryptoInfo_setPattern(
        info: *mut AMediaCodecCryptoInfo,
        pattern: *mut CryptoInfoPattern,
    );

    /// Since: API 21
    fn AMediaCodecCryptoInfo_getNumSubSamples(info: *mut AMediaCodecCryptoInfo) -> usize;

    /// Since: API 21
    fn AMediaCodecCryptoInfo_getKey(info: *mut AMediaCodecCryptoInfo, dst: *mut u8) -> isize;

    /// Since: API 21
    fn AMediaCodecCryptoInfo_getIV(info: *mut AMediaCodecCryptoInfo, dst: *mut u8) -> isize;

    /// Since: API 21
    fn AMediaCodecCryptoInfo_getMode(info: *mut AMediaCodecCryptoInfo) -> i32;

    /// Since: API 21
    fn AMediaCodecCryptoInfo_getClearBytes(
        info: *mut AMediaCodecCryptoInfo,
        dst: *mut usize,
    ) -> isize;

    /// Since: API 21
    fn AMediaCodecCryptoInfo_getEncryptedBytes(
        info: *mut AMediaCodecCryptoInfo,
        dst: *mut usize,
    ) -> isize;
}
// FFI FUNCTIONS END

/// This represents a buffer returned by mediacodec's input
/// This buffer should be filled with input data depending on whether the codec is an encoder or decoder
#[derive(Debug)]
pub struct CodecInputBuffer<'a> {
    _marker: PhantomData<&'a (*mut u8, core::marker::PhantomPinned)>,
    buffer: *mut u8,
    size: usize,
    write_size: usize,
    index: usize,
    codec: *mut AMediaCodec,
    time: u64,
    flags: u32,
    queued: bool,
}

impl CodecInputBuffer<'_> {
    /// Creates a new Codec Input Buffer from the parameters
    fn new(codec: *mut AMediaCodec, index: usize, buffer: *mut u8, size: usize) -> Self {
        Self {
            _marker: PhantomData,
            buffer,
            size,
            index,
            codec,
            write_size: 0,
            time: 0,
            flags: 0,
            queued: false,
        }
    }

    /// Returns this buffer's index. There's not much you can do with this
    pub fn index(&self) -> usize {
        self.index
    }

    /// The size (in bytes) of this buffer
    pub fn size(&self) -> usize {
        self.size
    }

    /// The presentation time of this buffer. If you did not set this yet, it will be zero
    pub fn time(&self) -> u64 {
        self.time
    }

    /// The size of data written into this buffer
    pub fn write_size(&self) -> usize {
        self.write_size
    }

    /// The buffer itself. It is returned as a mutable pointer
    ///
    /// The reason for this is that I find data copying primitives provided by Rust to be confusing and not performant,
    /// so I would recommend you just use the `copy_nonoverlapping` function to copy data to this pointer
    #[must_use]
    pub fn buffer(&self) -> (*mut u8, usize) {
        (self.buffer, self.size)
    }

    pub(crate) fn raw_buffer(&self) -> *mut u8 {
        self.buffer
    }

    pub(crate) fn raw_size(&self) -> usize {
        self.size
    }

    /// Set the presentation time of this buffer
    pub fn set_time(&mut self, time: u64) {
        self.time = time;
    }

    /// Set this buffer's flags
    pub fn set_flags(&mut self, flags: u32) {
        self.flags = flags;
    }

    /// Set the size of bytes written to this buffer
    pub fn set_write_size(&mut self, write_size: usize) {
        self.write_size = write_size;
    }

    /// Explicitly queue this buffer to the codec. After this, dropping the buffer is a no-op.
    pub fn queue(&mut self) {
        if !self.queued {
            self.queued = true;
            unsafe {
                AMediaCodec_queueInputBuffer(
                    self.codec,
                    self.index,
                    0,
                    self.write_size,
                    self.time,
                    self.flags,
                );
            }
        }
    }

    /// Cancel this buffer so it is NOT queued on drop. Use this on failure paths.
    pub fn cancel(&mut self) {
        self.queued = true;
    }
}

impl Drop for CodecInputBuffer<'_> {
    fn drop(&mut self) {
        if !self.queued {
            unsafe {
                AMediaCodec_queueInputBuffer(
                    self.codec,
                    self.index,
                    0,
                    self.write_size,
                    self.time,
                    self.flags,
                );
            }
        }
    }
}

unsafe impl Send for CodecInputBuffer<'_> {}

/// Represents a mediacodec output buffer
///
/// For decoders, this is a raw frame.
///
/// For encoders, this is an encoded packet
#[derive(Debug)]
pub struct CodecOutputBuffer<'a> {
    _marker: PhantomData<&'a (*mut u8, core::marker::PhantomPinned)>,
    codec: *mut AMediaCodec,
    info: BufferInfo,
    index: usize,
    using_buffers: bool,
    buffer: *mut u8,
    _size: usize,
    format: MediaFormat,
    render: bool,
}

impl CodecOutputBuffer<'_> {
    /// Create a new codec output buffer from the parameters
    fn new(
        codec: *mut AMediaCodec,
        info: BufferInfo,
        index: usize,
        using_buffers: bool,
        buffer: *mut u8,
        size: usize,
        format: MediaFormat,
    ) -> Self {
        Self {
            codec,
            info,
            index,
            using_buffers,
            buffer,
            _size: size,
            _marker: PhantomData,
            render: false,
            format,
        }
    }

    /// Returns the buffer information
    pub fn info(&self) -> &BufferInfo {
        &self.info
    }

    /// Returns the index of the codec buffer
    pub fn index(&self) -> usize {
        self.index
    }

    /// Whether we're returning raw buffers or using hardware buffers
    ///
    /// This only applies to video frames and a decoder
    pub fn using_buffers(&self) -> bool {
        self.using_buffers
    }

    /// The [MediaFormat](MediaFormat) associated with this buffer
    pub fn format(&self) -> &MediaFormat {
        &self.format
    }

    /// Returns the buffer as a u8 slice
    pub fn buffer_slice(&self) -> Option<&[u8]> {
        if !self.using_buffers {
            return None;
        }

        let offset = self.info.offset as usize;
        let size = self.info.size as usize;

        if offset + size > self._size {
            warn!(
                "Buffer offset ({}) + size ({}) exceeds capacity ({})",
                offset, size, self._size
            );
            return None;
        }

        Some(unsafe { slice::from_raw_parts(self.buffer.add(offset), size) })
    }

    /// Returns the frame contained in this buffer.
    /// Can either be an audio frame or a video frame
    pub fn frame(&self) -> Option<Frame<'_>> {
        // Determine whether this is an audio or video frame.
        // We can use the mime type to do this
        let mime = self.format.get_string("mime")?;
        let is_audio: bool;

        // We don't know if we might get some weird mime types, so we check for both audio and video explicitly
        if mime.contains("audio") {
            is_audio = true;
        } else if mime.contains("video") {
            is_audio = false;
        } else {
            debug!("Mime is not a valid one!");
            return None;
        }

        if is_audio {
            // Fetch the PCM Encoding
            let encoding = self.format.get_i32("pcm-encoding")?;
            let channels = self.format.get_i32("channel-count")?;

            // Can't have invalid channels!
            if channels <= 0 {
                debug!("Channels <= 0!");
                return None;
            }

            match encoding {
                ENCODING_PCM_16BIT => {
                    let slice = self.buffer_slice()?;
                    let byte_len = slice.len();
                    let sample_count = byte_len / std::mem::size_of::<i16>();

                    if sample_count == 0 {
                        return None;
                    }

                    let cow: Cow<'_, [i16]> =
                        if slice.as_ptr() as usize % std::mem::align_of::<i16>() == 0 {
                            Cow::Borrowed(unsafe {
                                slice::from_raw_parts(slice.as_ptr() as *const i16, sample_count)
                            })
                        } else {
                            let mut aligned = vec![0i16; sample_count];
                            unsafe {
                                std::ptr::copy_nonoverlapping(
                                    slice.as_ptr(),
                                    aligned.as_mut_ptr() as *mut u8,
                                    sample_count * std::mem::size_of::<i16>(),
                                );
                            }
                            Cow::Owned(aligned)
                        };

                    Some(Frame::Audio(AudioFrame::new(
                        SampleFormat::S16(cow),
                        channels as u32,
                    )))
                }
                ENCODING_PCM_FLOAT => {
                    let slice = self.buffer_slice()?;
                    let byte_len = slice.len();
                    let sample_count = byte_len / std::mem::size_of::<f32>();

                    if sample_count == 0 {
                        return None;
                    }

                    let cow: Cow<'_, [f32]> =
                        if slice.as_ptr() as usize % std::mem::align_of::<f32>() == 0 {
                            Cow::Borrowed(unsafe {
                                slice::from_raw_parts(slice.as_ptr() as *const f32, sample_count)
                            })
                        } else {
                            let mut aligned = vec![0f32; sample_count];
                            unsafe {
                                std::ptr::copy_nonoverlapping(
                                    slice.as_ptr(),
                                    aligned.as_mut_ptr() as *mut u8,
                                    sample_count * std::mem::size_of::<f32>(),
                                );
                            }
                            Cow::Owned(aligned)
                        };

                    Some(Frame::Audio(AudioFrame::new(
                        SampleFormat::F32(cow),
                        channels as u32,
                    )))
                }
                _ => None,
            }
        } else {
            // We have a video frame! Do justice to it

            if !self.using_buffers {
                Some(Frame::Video(VideoFrame::Hardware))
            } else {
                None
            }
        }
    }

    /// Set whether this buffer should render when it gets dropped.
    /// This only works for video decoder buffers with a surface attached
    pub fn set_render(&mut self, render: bool) {
        self.render = render;
    }
}

impl Drop for CodecOutputBuffer<'_> {
    fn drop(&mut self) {
        unsafe {
            AMediaCodec_releaseOutputBuffer(self.codec, self.index, self.render);
        }
    }
}

unsafe impl Send for CodecOutputBuffer<'_> {}

/// The MediaCodec structure itself.
///
/// Represents either a decoder or an encoder
#[derive(Debug)]
pub struct MediaCodec {
    inner: *mut AMediaCodec,
    using_buffers: bool,
    output_format: Option<MediaFormat>,
}

impl MediaCodec {
    /// Creates a MediaCodec instance from raw pointer
    fn from_ptr(ptr: *mut AMediaCodec) -> Self {
        Self {
            inner: ptr,
            using_buffers: false,
            output_format: None,
        }
    }

    /// Creates a codec using its name
    pub fn new(name: &str) -> Result<Self, MediaStatus> {
        unsafe {
            let name = CString::new(name).map_err(|_| MediaStatus::ErrorInvalidParameter)?;
            let codec = AMediaCodec_createCodecByName(name.as_ptr());

            if codec.is_null() {
                return Err(MediaStatus::ErrorUnknown);
            }

            Ok(Self::from_ptr(codec))
        }
    }

    /// Creates a decoder using a specific mime type
    pub fn create_decoder(mime_type: &str) -> Result<Self, MediaStatus> {
        unsafe {
            let mime_type =
                CString::new(mime_type).map_err(|_| MediaStatus::ErrorInvalidParameter)?;
            let codec = AMediaCodec_createDecoderByType(mime_type.as_ptr());

            if codec.is_null() {
                return Err(MediaStatus::ErrorUnknown);
            }

            Ok(Self::from_ptr(codec))
        }
    }

    /// Creates an encoder using a specific mime type
    pub fn create_encoder(mime_type: &str) -> Result<Self, MediaStatus> {
        unsafe {
            let mime_type =
                CString::new(mime_type).map_err(|_| MediaStatus::ErrorInvalidParameter)?;
            let codec = AMediaCodec_createEncoderByType(mime_type.as_ptr());

            if codec.is_null() {
                return Err(MediaStatus::ErrorUnknown);
            }

            Ok(Self::from_ptr(codec))
        }
    }

    /// Initializes the codec with the parameters. This must be called before you can start the codec
    pub fn init(
        &mut self,
        format: &MediaFormat,
        surface: Option<&NativeWindow>,
        flags: u32,
    ) -> Result<(), MediaStatus> {
        unsafe {
            // Keep the NativeWindow alive until after AMediaCodec_configure
            let surface_ptr = surface.map_or(null_mut(), |s| s.inner);
            self.using_buffers = surface.is_none();

            MediaStatus::make_result(AMediaCodec_configure(
                self.inner,
                format.inner,
                surface_ptr,
                null_mut(),
                flags,
            ))
        }
    }

    /// Starts the codec for processing.
    ///
    /// This must be called only after the codec has been initialized
    pub fn start(&mut self) -> Result<(), MediaStatus> {
        unsafe { MediaStatus::make_result(AMediaCodec_start(self.inner)) }
    }

    /// **WARNING**
    ///
    /// Make sure you have released all pending buffers before calling this function
    pub fn stop(&mut self) -> Result<(), MediaStatus> {
        unsafe { MediaStatus::make_result(AMediaCodec_stop(self.inner)) }
    }

    /// **WARNING**
    ///
    /// Make sure you have released all pending buffers before calling this function
    pub fn flush(&mut self) -> Result<(), MediaStatus> {
        unsafe { MediaStatus::make_result(AMediaCodec_flush(self.inner)) }
    }

    /// Returns the output format of this codec (if we can find one)
    #[must_use]
    pub fn output_format(&self) -> Option<MediaFormat> {
        unsafe {
            let format = AMediaCodec_getOutputFormat(self.inner);
            if format.is_null() {
                return None;
            }

            Some(MediaFormat::from_raw(format))
        }
    }

    fn refresh_output_format(&mut self) -> Option<MediaFormat> {
        let format = self.output_format();
        self.output_format = format.clone();
        format
    }

    /// Sets the codec output surface. This will only work if the codec has been initialized with an output surface
    /// before starting
    pub fn set_output_surface(&mut self, window: &NativeWindow) -> Result<(), MediaStatus> {
        if self.using_buffers {
            return Err(MediaStatus::ErrorInvalidOperation);
        }

        unsafe { MediaStatus::make_result(AMediaCodec_setOutputSurface(self.inner, window.inner)) }
    }

    /// Get an input buffer from mediacodec
    ///
    /// `timeout_us` is the timeout in microseconds. Use `-1` to block indefinitely.
    pub fn dequeue_input(&mut self, timeout_us: i64) -> Result<CodecInputBuffer<'_>, DequeueInputError> {
        unsafe {
            let index = AMediaCodec_dequeueInputBuffer(self.inner, timeout_us);

            if index >= 0 {
                let mut out_size = 0;
                let buffer = AMediaCodec_getInputBuffer(self.inner, index as usize, &mut out_size);

                if buffer.is_null() {
                    // Return the buffer to the codec, it's not valid
                    AMediaCodec_queueInputBuffer(self.inner, index as usize, 0, 0, 0, 0);
                    warn!(
                        "Got an index with a null input buffer! What is going on here??? Index: {index}"
                    );
                    return Err(DequeueInputError::CodecError(MediaStatus::ErrorUnknown));
                }

                Ok(CodecInputBuffer::new(
                    self.inner,
                    index as usize,
                    buffer,
                    out_size,
                ))
            } else if index == -1 {
                Err(DequeueInputError::TryAgainLater)
            } else {
                Err(DequeueInputError::CodecError(
                    MediaStatus::from_i32(index as i32),
                ))
            }
        }
    }

    /// Get an output buffer from mediacodec
    ///
    /// `timeout_us` is the timeout in microseconds. Use `-1` to block indefinitely.
    pub fn dequeue_output(&mut self, timeout_us: i64) -> Result<CodecOutputBuffer<'_>, DequeueOutputError> {
        unsafe {
            let mut info = BufferInfo::default();
            let index = AMediaCodec_dequeueOutputBuffer(self.inner, &mut info, timeout_us);

            match index {
                -1 => Err(DequeueOutputError::TryAgainLater),
                -2 => {
                    self.refresh_output_format();
                    Err(DequeueOutputError::OutputFormatChanged)
                }
                -3 => Err(DequeueOutputError::OutputBuffersChanged),
                idx if idx >= 0 => {
                    let mut out_size = 0;
                    let mut buffer = null_mut();
                    if self.using_buffers {
                        buffer =
                            AMediaCodec_getOutputBuffer(self.inner, idx as usize, &mut out_size);

                        if buffer.is_null() {
                            AMediaCodec_releaseOutputBuffer(self.inner, idx as usize, false);
                            return Err(DequeueOutputError::CodecError(MediaStatus::ErrorUnknown));
                        }
                    }

                    let format = self
                        .output_format
                        .clone()
                        .or_else(|| self.refresh_output_format())
                        .ok_or(DequeueOutputError::CodecError(MediaStatus::ErrorUnknown))?;

                    Ok(CodecOutputBuffer::new(
                        self.inner,
                        info,
                        idx as usize,
                        self.using_buffers,
                        buffer,
                        out_size,
                        format,
                    ))
                }
                _ => Err(DequeueOutputError::CodecError(
                    MediaStatus::from_i32(index as i32),
                )),
            }
        }
    }
}

impl Drop for MediaCodec {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                AMediaCodec_delete(self.inner);
            }
        }
    }
}

unsafe impl Send for MediaCodec {}
