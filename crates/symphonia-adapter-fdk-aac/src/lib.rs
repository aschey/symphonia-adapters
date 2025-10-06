#![warn(missing_docs)]
#![forbid(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

mod adts;
mod meta;

use fdk_aac::dec::{Decoder, DecoderError, Transport};
use log::warn;
use symphonia_core::audio::{AsAudioBufferRef, AudioBuffer, AudioBufferRef, Signal, SignalSpec};
use symphonia_core::codecs::{
    CODEC_TYPE_AAC, CodecDescriptor, CodecParameters, DecoderOptions, FinalizeResult,
};
use symphonia_core::errors::{Error, unsupported_error};
use symphonia_core::formats::Packet;
use symphonia_core::support_codec;

use crate::adts::construct_adts_header;
use crate::macros::validate;
use crate::meta::{M4A_TYPES, M4AInfo, M4AType, map_to_channels, sample_rate_index};

type Result<T> = symphonia_core::errors::Result<T>;

mod macros {
    macro_rules! validate {
        ($a:expr) => {
            if !$a {
                log::error!("check failed at {}:{}", file!(), line!());
                return symphonia_core::errors::decode_error("aac: invalid data");
            }
        };
    }
    pub(crate) use validate;
}

const MAX_SAMPLES: usize = 8192;

/// Symphonia-compatible wrapper for the FDK AAC decoder.
pub struct AacDecoder {
    decoder: Decoder,
    buf: AudioBuffer<i16>,
    codec_params: CodecParameters,
    m4a_info: M4AInfo,
    m4a_info_validated: bool,
    pcm: [i16; MAX_SAMPLES],
}

impl AacDecoder {
    fn configure_metadata(&mut self) -> Result<()> {
        let stream_info = self.decoder.stream_info();
        let capacity = self.decoder.decoded_frame_size();
        let channels = stream_info.numChannels as usize;
        let sample_rate = stream_info.aacSampleRate as u32;

        self.m4a_info = M4AInfo {
            otype: M4A_TYPES[stream_info.aot as usize],
            channels: stream_info.numChannels as u8,
            sample_rate,
            sample_rate_index: sample_rate_index(sample_rate),
            samples: capacity / channels,
        };

        self.buf = audio_buffer(&self.m4a_info, stream_info.sampleRate as u32)?;
        self.m4a_info_validated = true;

        Ok(())
    }
}

fn audio_buffer(m4a_info: &M4AInfo, sample_rate: u32) -> Result<AudioBuffer<i16>> {
    if m4a_info.channels < 1 || m4a_info.channels > 2 {
        return unsupported_error("aac: unsupported number of channels");
    }
    let channels = map_to_channels(m4a_info.channels).expect("invalid channels");

    let spec = SignalSpec::new(sample_rate, channels);
    Ok(AudioBuffer::new(m4a_info.samples as u64, spec))
}

impl symphonia_core::codecs::Decoder for AacDecoder {
    fn try_new(params: &CodecParameters, _opts: &DecoderOptions) -> Result<Self> {
        let mut m4a_info = M4AInfo::default();
        if let Some(extra_data_buf) = &params.extra_data {
            validate!(extra_data_buf.len() >= 2);
            m4a_info.read(extra_data_buf)?;
        } else {
            m4a_info.otype = M4AType::Lc;
            m4a_info.sample_rate = params.sample_rate.unwrap_or_default();
            m4a_info.sample_rate_index = sample_rate_index(m4a_info.sample_rate);

            m4a_info.channels = if let Some(channels) = &params.channels {
                channels.count() as u8
            } else {
                return unsupported_error("aac: channels or channel layout is required");
            };
        }
        let decoder = Decoder::new(Transport::Adts);

        let buf = audio_buffer(&m4a_info, m4a_info.sample_rate)?;
        Ok(Self {
            decoder,
            codec_params: params.clone(),
            buf,
            m4a_info,
            // We should always prefer the m4a info from the decoder even if we were able to parse
            // the extra data from the header since it could be more accurate
            m4a_info_validated: false,
            pcm: [0; _],
        })
    }

    fn reset(&mut self) {}

    fn supported_codecs() -> &'static [CodecDescriptor] {
        &[support_codec!(
            CODEC_TYPE_AAC,
            "aac",
            "Advanced Audio Coding"
        )]
    }

    fn codec_params(&self) -> &CodecParameters {
        &self.codec_params
    }

    fn decode(&mut self, packet: &Packet) -> Result<AudioBufferRef<'_>> {
        let adts_header = construct_adts_header(
            self.m4a_info.otype,
            self.m4a_info.sample_rate_index,
            self.m4a_info.channels,
            packet.buf().len(),
        );
        self.decoder
            .fill(&[&adts_header, packet.buf()].concat())
            .map_err(|e| Error::DecodeError(e.message()))?;

        match self.decoder.decode_frame(&mut self.pcm) {
            Ok(_) => {}
            Err(e @ DecoderError::TRANSPORT_SYNC_ERROR) => {
                warn!("aac: transport sync error: {}", e.message());
                self.buf.clear();
                return Ok(self.buf.as_audio_buffer_ref());
            }
            Err(e) => {
                return Err(Error::DecodeError(e.message()));
            }
        }
        if !self.m4a_info_validated {
            self.configure_metadata()?;
        }

        let capacity = self.decoder.decoded_frame_size();
        let pcm = &self.pcm[..capacity];
        self.buf.clear();

        self.buf.render_reserved(None);
        match self.m4a_info.channels {
            1 => {
                self.buf.chan_mut(0).copy_from_slice(pcm);
            }
            2 => {
                let (l, r) = self.buf.chan_pair_mut(0, 1);
                for (i, j) in (0..capacity).step_by(2).enumerate() {
                    l[i] = pcm[j];
                    r[i] = pcm[j + 1];
                }
            }
            _ => {}
        }

        self.buf
            .trim(packet.trim_start() as usize, packet.trim_end() as usize);
        Ok(self.buf.as_audio_buffer_ref())
    }

    fn finalize(&mut self) -> FinalizeResult {
        FinalizeResult::default()
    }

    fn last_decoded(&self) -> AudioBufferRef<'_> {
        self.buf.as_audio_buffer_ref()
    }
}
