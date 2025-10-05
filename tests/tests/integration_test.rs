use std::fs::File;

use symphonia::core::codecs::CodecParameters;
use symphonia::core::codecs::registry::CodecRegistry;
use symphonia::core::formats::TrackType;
use symphonia::core::formats::probe::Hint;
use symphonia::core::io::MediaSourceStream;
use symphonia::default::get_probe;
use symphonia_adapter_fdk_aac::AacDecoder;
use symphonia_adapter_libopus::OpusDecoder;

#[test]
fn test_decode_aac() {
    test_decode(File::open("../assets/music.m4a").unwrap());
}

#[test]
fn test_decode_opus() {
    test_decode(File::open("../assets/sample.opus").unwrap());
}

fn test_decode(file: File) {
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut reader = get_probe()
        .probe(&Hint::new(), mss, Default::default(), Default::default())
        .unwrap();
    let mut registry = CodecRegistry::new();
    registry.register_audio_decoder::<AacDecoder>();
    registry.register_audio_decoder::<OpusDecoder>();

    let track = reader.default_track(TrackType::Audio).unwrap();
    let track_id = track.id;
    let Some(CodecParameters::Audio(codec_params)) = track.codec_params.as_ref() else {
        panic!("invalid params");
    };
    let mut decoder = registry
        .make_audio_decoder(codec_params, &Default::default())
        .unwrap();

    loop {
        let Some(packet) = reader.next_packet().unwrap() else {
            break;
        };

        if packet.track_id() != track_id {
            continue;
        }
        decoder.decode(&packet).map(|_| ()).unwrap();
    }
}
