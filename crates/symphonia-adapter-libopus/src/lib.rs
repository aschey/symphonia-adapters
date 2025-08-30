use symphonia_core::audio::{
    AsGenericAudioBufferRef, AudioBuffer, AudioMut, AudioSpec, Channels, GenericAudioBufferRef,
    layouts,
};
use symphonia_core::codecs::CodecInfo;
use symphonia_core::codecs::audio::well_known::CODEC_ID_OPUS;
use symphonia_core::codecs::audio::{
    AudioCodecParameters, AudioDecoder, AudioDecoderOptions, FinalizeResult,
};
use symphonia_core::codecs::registry::{RegisterableAudioDecoder, SupportedAudioCodec};
use symphonia_core::errors::{Result, unsupported_error};
use symphonia_core::support_audio_codec;

use crate::decoder::Decoder;

mod decoder;

const DEFAULT_SAMPLES_PER_CHANNEL: usize = 960;

pub struct OpusDecoder {
    params: AudioCodecParameters,
    decoder: Decoder,
    buf: AudioBuffer<i16>,
    pcm: [i16; DEFAULT_SAMPLES_PER_CHANNEL * 2],
    samples_per_channel: usize,
    sample_rate: u32,
    num_channels: usize,
}

impl OpusDecoder {
    fn try_new(
        params: &AudioCodecParameters,
        _opts: &AudioDecoderOptions,
    ) -> symphonia_core::errors::Result<Self>
    where
        Self: Sized,
    {
        println!("{params:?}");
        let num_channels = if let Some(channels) = &params.channels {
            channels.count()
        } else {
            return unsupported_error("opus: channels or channel layout is required");
        };
        let sample_rate = if let Some(sample_rate) = params.sample_rate {
            sample_rate
        } else {
            return unsupported_error("opus: sample rate required");
        };

        if !(1..=2).contains(&num_channels) {
            return unsupported_error("aac: unsupported number of channels");
        }

        Ok(Self {
            params: params.to_owned(),
            decoder: Decoder::new(sample_rate, num_channels as u32)?,
            buf: audio_buffer(sample_rate, DEFAULT_SAMPLES_PER_CHANNEL, num_channels),
            pcm: [0; _],
            samples_per_channel: DEFAULT_SAMPLES_PER_CHANNEL,
            sample_rate,
            num_channels,
        })
    }
}

impl symphonia_core::codecs::audio::AudioDecoder for OpusDecoder {
    fn codec_info(&self) -> &CodecInfo {
        &Self::supported_codecs()
            .first()
            .expect("missing codecs")
            .info
    }

    fn reset(&mut self) {}

    fn codec_params(&self) -> &AudioCodecParameters {
        &self.params
    }

    fn decode(
        &mut self,
        packet: &symphonia_core::formats::Packet,
    ) -> Result<GenericAudioBufferRef<'_>> {
        let samples_per_channel = self.decoder.decode(&packet.data, &mut self.pcm)?;

        if samples_per_channel != self.samples_per_channel {
            self.buf = audio_buffer(self.sample_rate, samples_per_channel, self.num_channels);
            self.samples_per_channel = samples_per_channel;
        }

        let samples = samples_per_channel * self.num_channels;
        let pcm = &self.pcm[..samples];

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

impl RegisterableAudioDecoder for OpusDecoder {
    fn try_registry_new(
        params: &AudioCodecParameters,
        opts: &AudioDecoderOptions,
    ) -> Result<Box<dyn AudioDecoder>>
    where
        Self: Sized,
    {
        Ok(Box::new(OpusDecoder::try_new(params, opts)?))
    }

    fn supported_codecs() -> &'static [SupportedAudioCodec] {
        &[support_audio_codec!(CODEC_ID_OPUS, "opus", "Opus")]
    }
}

pub(crate) fn map_to_channels(num_channels: usize) -> Option<Channels> {
    let channels = match num_channels {
        1 => layouts::CHANNEL_LAYOUT_MONO,
        2 => layouts::CHANNEL_LAYOUT_STEREO,
        3 => layouts::CHANNEL_LAYOUT_AAC_3P0,
        4 => layouts::CHANNEL_LAYOUT_AAC_4P0,
        5 => layouts::CHANNEL_LAYOUT_AAC_5P0,
        6 => layouts::CHANNEL_LAYOUT_AAC_5P1,
        8 => layouts::CHANNEL_LAYOUT_AAC_7P1,
        _ => return None,
    };

    Some(channels)
}

fn audio_buffer(
    sample_rate: u32,
    samples_per_channel: usize,
    num_channels: usize,
) -> AudioBuffer<i16> {
    let channels = map_to_channels(num_channels).expect("invalid channels");
    let spec = AudioSpec::new(sample_rate, channels);
    AudioBuffer::new(spec, samples_per_channel)
}

