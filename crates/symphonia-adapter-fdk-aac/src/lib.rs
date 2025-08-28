mod adts;
mod meta;

use fdk_aac::dec::{Decoder, DecoderError, Transport};
use symphonia_core::audio::{
    AsGenericAudioBufferRef, AudioBuffer, AudioMut, AudioSpec, GenericAudioBufferRef,
};
use symphonia_core::codecs::CodecInfo;
use symphonia_core::codecs::audio::well_known::CODEC_ID_AAC;
use symphonia_core::codecs::audio::well_known::profiles::{
    CODEC_PROFILE_AAC_HE, CODEC_PROFILE_AAC_HE_V2,
};
use symphonia_core::codecs::audio::{
    AudioCodecParameters, AudioDecoder, AudioDecoderOptions, FinalizeResult,
};
use symphonia_core::codecs::registry::{RegisterableAudioDecoder, SupportedAudioCodec};
use symphonia_core::errors::{Error, unsupported_error};
use symphonia_core::formats::Packet;
use symphonia_core::{codec_profile, support_audio_codec};
use tracing::warn;

use crate::adts::construct_adts_header;
use crate::macros::validate;
use crate::meta::{M4A_TYPES, M4AInfo, M4AType, map_to_channels, sample_rate_index};

type Result<T> = symphonia_core::errors::Result<T>;

mod macros {
    macro_rules! validate {
        ($a:expr) => {
            if !$a {
                tracing::error!("check failed at {}:{}", file!(), line!());
                return symphonia_core::errors::decode_error("aac: invalid data");
            }
        };
    }
    pub(crate) use validate;
}

pub struct AacDecoder {
    decoder: Decoder,
    buf: AudioBuffer<i16>,
    codec_params: AudioCodecParameters,
    m4a_info: M4AInfo,
    m4a_info_validated: bool,
    pcm: [i16; 8192],
}

impl AacDecoder {
    pub fn new(params: &AudioCodecParameters, _opts: &AudioDecoderOptions) -> Result<Self> {
        let mut m4a_info = M4AInfo::default();
        let mut m4a_info_validated = false;
        if let Some(extra_data_buf) = &params.extra_data {
            validate!(extra_data_buf.len() >= 2);
            m4a_info.read(extra_data_buf)?;
            m4a_info_validated = true;
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
            m4a_info_validated,
            pcm: [0; 8192],
        })
    }

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
    Ok(AudioBuffer::new(
        AudioSpec::new(sample_rate, channels),
        m4a_info.samples,
    ))
}

impl AudioDecoder for AacDecoder {
    fn reset(&mut self) {}

    fn codec_info(&self) -> &CodecInfo {
        &Self::supported_codecs()
            .first()
            .expect("missing codecs")
            .info
    }

    fn codec_params(&self) -> &AudioCodecParameters {
        &self.codec_params
    }

    fn decode(&mut self, packet: &Packet) -> Result<GenericAudioBufferRef<'_>> {
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
                return Ok(self.buf.as_generic_audio_buffer_ref());
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

        self.buf.render_uninit(None);
        self.buf.copy_from_slice_interleaved(&pcm);
        Ok(self.buf.as_generic_audio_buffer_ref())
    }

    fn finalize(&mut self) -> FinalizeResult {
        FinalizeResult::default()
    }

    fn last_decoded(&self) -> GenericAudioBufferRef<'_> {
        self.buf.as_generic_audio_buffer_ref()
    }
}

impl RegisterableAudioDecoder for AacDecoder {
    fn try_registry_new(
        params: &AudioCodecParameters,
        opts: &AudioDecoderOptions,
    ) -> symphonia_core::errors::Result<Box<dyn AudioDecoder>>
    where
        Self: Sized,
    {
        Ok(Box::new(AacDecoder::new(params, opts)?))
    }

    fn supported_codecs() -> &'static [SupportedAudioCodec] {
        use symphonia_core::codecs::audio::well_known::profiles::CODEC_PROFILE_AAC_LC;

        &[support_audio_codec!(
            CODEC_ID_AAC,
            "aac",
            "Advanced Audio Coding",
            &[
                codec_profile!(CODEC_PROFILE_AAC_LC, "aac-lc", "Low Complexity"),
                codec_profile!(CODEC_PROFILE_AAC_HE, "aac-he", "High Efficiency"),
                codec_profile!(CODEC_PROFILE_AAC_HE_V2, "aac-he-v2", "High Efficiency V2"),
            ]
        )]
    }
}
