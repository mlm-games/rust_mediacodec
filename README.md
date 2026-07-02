# Rust MediaCodec
This library provides Rust bindings to the Android MediaCodec APIs. It also adds some pretty nifty utilities to make working with buffers on Android easier.

## Features Currently Implemented
- [x] MediaCodec
- [x] MediaExtractor
- [x] MediaMuxer
- [x] MediaFormat
- [x] Safe codec buffers abstraction
- [x] Some extra utilities to make working with the library easier

## Feature flags

- `api24` — no additional APIs
- `api26` — `createInputSurface`, `createPersistentInputSurface`, `setInputSurface`, `setParameters`, `signalEndOfInputStream`
- `api28` — `getBufferFormat`, `getName`, `setAsyncNotifyCallback`, `releaseCrypto`, `getInputFormat`, `AMediaCodecActionCode_isRecoverable`, `AMediaCodecActionCode_isTransient`, `getDouble`/`setDouble`/`getRect`/`setSize`/`setRect` on `MediaFormat`
- `api29` — `clear`/`copy` on `MediaFormat`

## Decoding example

```rust
use log::debug;
use mediacodec::{DequeueInputError, DequeueOutputError, Frame, MediaCodec, MediaExtractor, SampleFormat, VideoFrame};

#[unsafe(no_mangle)]
extern "C" fn process() {
    let mut extractor = MediaExtractor::from_url("/path/to/a/resource").unwrap();

    debug!("Track count: {}", extractor.track_count());

    let mut decoders = vec![];

    for i in 0..extractor.track_count() {
        let format = extractor.track_format(i).unwrap();
        debug!("{}", format.to_string());
        let mime_type = format.get_string("mime").unwrap();
        let mut codec = MediaCodec::create_decoder(&mime_type).unwrap();

        codec.init(&format, None, 0).unwrap();
        codec.start().unwrap();
        decoders.push(codec);
        extractor.select_track(i).unwrap();
    }

    while extractor.has_next() {
        let index = extractor.track_index();
        if index < 0 {
            break;
        }

        let codec = &mut decoders[index as usize];

        loop {
            match codec.dequeue_input(100) {
                Ok(mut buffer) => {
                    match extractor.read_next(&mut buffer) {
                        Ok(true) => {}
                        Ok(false) => {
                            buffer.cancel();
                            break;
                        }
                        Err(_) => {
                            buffer.cancel();
                            break;
                        }
                    }
                }
                Err(DequeueInputError::TryAgainLater) => break,
                Err(DequeueInputError::CodecError(e)) => {
                    debug!("Codec error: {e:?}");
                    break;
                }
            }

            // When the buffer gets dropped (here), the buffer will be queued back to MediaCodec
            // And we don't have to do anything else
        }

        loop {
            match codec.dequeue_output(100) {
                Ok(mut buffer) => {
                    if let Some(ref frame) = buffer.frame() {
                        match frame {
                            Frame::Audio(value) => match value.format() {
                                SampleFormat::S16(_) => {}
                                SampleFormat::F32(_) => {}
                            },
                            Frame::Video(value) => match value {
                                VideoFrame::Hardware => {}
                                VideoFrame::RawFrame(_) => {}
                            },
                        }
                    }
                    buffer.set_render(true);
                }
                Err(DequeueOutputError::TryAgainLater) => break,
                Err(DequeueOutputError::OutputFormatChanged) => {
                    debug!("Output format changed");
                    continue;
                }
                Err(DequeueOutputError::OutputBuffersChanged) => continue,
                Err(DequeueOutputError::CodecError(e)) => {
                    debug!("Codec error: {e:?}");
                    break;
                }
            }
        }
    }
}
```

You can find more examples in the **examples** directory.
