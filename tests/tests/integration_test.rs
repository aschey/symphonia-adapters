use std::fs::File;

use symphonia::core::codecs::CodecRegistry;
use symphonia::core::errors::{Error, Result};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;
use symphonia_adapter_fdk_aac::AacDecoder;
use symphonia_adapter_libopus::OpusDecoder;

#[test]
fn test_decode_aac() {
    test_decode(File::open("./assets/music.m4a").unwrap());
}

#[test]
fn test_decode_opus() {
    test_decode(File::open("./assets/sample.opus").unwrap());
}

fn test_decode(file: File) {
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let probe = get_probe()
        .format(&Hint::new(), mss, &Default::default(), &Default::default())
        .unwrap();
    let mut registry = CodecRegistry::new();
    registry.register_all::<AacDecoder>();
    registry.register_all::<OpusDecoder>();

    let mut reader = probe.format;
    let track = reader.default_track().unwrap();
    let track_id = track.id;
    let mut decoder = registry
        .make(&track.codec_params, &Default::default())
        .unwrap();

    loop {
        let packet_res = reader.next_packet();
        if is_end_of_stream_error(&packet_res) {
            break;
        }
        let packet = packet_res.unwrap();
        if packet.track_id() != track_id {
            continue;
        }
        decoder.decode(&packet).map(|_| ()).unwrap();
    }
}

fn is_end_of_stream_error<T>(result: &Result<T>) -> bool {
    match result {
        Err(Error::IoError(err))
            if err.kind() == std::io::ErrorKind::UnexpectedEof
                && err.to_string() == "end of stream" =>
        {
            // Do not treat "end of stream" as a fatal error. It's the currently only way a
            // format reader can indicate the media is complete.
            true
        }
        _ => false,
    }
}
