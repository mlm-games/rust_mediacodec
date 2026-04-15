//! Rust bindings to the MediaCodec APIs in the Android NDK.
//!
//! # Examples
//!
//! ### Decoding
//! ```
//! # use mediacodec::{Frame, MediaCodec, MediaExtractor, SampleFormat, VideoFrame};
//! # #[unsafe(no_mangle)]
//! # extern "C" fn process() {
//!     let mut extractor = MediaExtractor::from_url("/path/to/a/resource").unwrap();
//!     // ... (see the `examples/decoding.rs` file for the full example)
//! # }
//! ```
//!
//! ### Demuxing
//! ```
//! # use mediacodec::MediaExtractor;
//! # #[unsafe(no_mangle)]
//! # extern "C" fn process() {
//!     let mut extractor = MediaExtractor::from_url("/path/to/a/resource").unwrap();
//!     // ... (see the `examples/demuxing.rs` file for the full example)
//! # }
//! ```

mod codec;
mod crypto;
mod error;
mod extractor;
mod format;
mod muxer;
mod native_window;
mod samples;

pub use codec::*;
pub use crypto::*;
pub use error::*;
pub use extractor::*;
pub use format::*;
pub use muxer::*;
pub use native_window::*;
pub use samples::*;
