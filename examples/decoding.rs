use log::debug;
use mediacodec::{
    DequeueInputError, DequeueOutputError, Frame, MediaCodec, MediaExtractor, SampleFormat,
    VideoFrame,
};

#[unsafe(no_mangle)]
extern "C" fn process() {
    let mut extractor = MediaExtractor::from_url("/path/to/a/resource").unwrap();

    debug!("Track count: {}", extractor.track_count());

    let mut decoders = vec![];

    for i in 0..extractor.track_count() {
        let format = extractor.track_format(i).unwrap();
        debug!("{}", format.to_string());
        let mime_type = format.get_string("mime").unwrap();
        let mut codec = MediaCodec::create_decoder(&mime_type).expect("failed to create decoder");

        codec.init(&format, None, 0).unwrap();

        codec.start().unwrap();
        decoders.push(codec);
        extractor.select_track(i).unwrap();
    }

    while extractor.has_next() {
        // 1. Get the track index
        let index = extractor.track_index();

        if index < 0 {
            break;
        }

        let codec = &mut decoders[index as usize];

        // Fetch the codec's input buffer
        loop {
            match codec.dequeue_input(100) {
                Ok(mut buffer) => {
                    match extractor.read_next(&mut buffer) {
                        Ok(true) => {}
                        Ok(false) => {
                            debug!(
                                "MediaExtractor.read_next() returned false! has_next(): {}",
                                extractor.has_next()
                            );
                            buffer.cancel();
                            break;
                        }
                        Err(_) => {
                            buffer.cancel();
                            break;
                        }
                    }
                    // buffer is queued on drop
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
