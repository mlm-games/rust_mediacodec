use log::debug;
use mediacodec::MediaExtractor;

#[unsafe(no_mangle)]
extern "C" fn process() {
    let mut extractor = MediaExtractor::from_url("/path/to/a/resource").unwrap();

    debug!("Track count: {}", extractor.track_count());

    for i in 0..extractor.track_count() {
        let format = extractor.track_format(i).unwrap();
        debug!("{}", format.to_string());
        let _mime_type = format.get_string("mime").unwrap();
        extractor.select_track(i).unwrap();
    }

    while extractor.has_next() {
        let _index = extractor.track_index();
    }
}
