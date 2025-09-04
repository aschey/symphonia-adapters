use symphonia_core::audio::{
    AsAudioBufferRef, AudioBuffer, AudioBufferRef, Channels, Layout, Signal, SignalSpec,
};
use symphonia_core::codecs::{CODEC_TYPE_OPUS, CodecDescriptor, CodecParameters, FinalizeResult};
use symphonia_core::errors::{Result, unsupported_error};
use symphonia_core::support_codec;

use crate::decoder::Decoder;

mod decoder;

const DEFAULT_SAMPLES_PER_CHANNEL: usize = 960;
const MAX_SAMPLES_PER_CHANNEL: usize = 2880;

pub struct OpusDecoder {
    params: CodecParameters,
    decoder: Decoder,
    buf: AudioBuffer<i16>,
    pcm: [i16; MAX_SAMPLES_PER_CHANNEL * 2],
    samples_per_channel: usize,
    sample_rate: u32,
    num_channels: usize,
}

impl symphonia_core::codecs::Decoder for OpusDecoder {
    fn try_new(
        params: &symphonia_core::codecs::CodecParameters,
        _opts: &symphonia_core::codecs::DecoderOptions,
    ) -> symphonia_core::errors::Result<Self>
    where
        Self: Sized,
    {
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
            buf: audio_buffer(
                sample_rate,
                DEFAULT_SAMPLES_PER_CHANNEL as u64,
                num_channels,
            ),
            pcm: [0; _],
            samples_per_channel: DEFAULT_SAMPLES_PER_CHANNEL,
            sample_rate,
            num_channels,
        })
    }

    fn supported_codecs() -> &'static [symphonia_core::codecs::CodecDescriptor]
    where
        Self: Sized,
    {
        &[support_codec!(CODEC_TYPE_OPUS, "opus", "Opus")]
    }

    fn reset(&mut self) {}

    fn codec_params(&self) -> &symphonia_core::codecs::CodecParameters {
        &self.params
    }

    fn decode(&mut self, packet: &symphonia_core::formats::Packet) -> Result<AudioBufferRef<'_>> {
        let samples_per_channel = self.decoder.decode(&packet.data, &mut self.pcm)?;

        if samples_per_channel != self.samples_per_channel {
            self.buf = audio_buffer(
                self.sample_rate,
                samples_per_channel as u64,
                self.num_channels,
            );
            self.samples_per_channel = samples_per_channel;
        }

        let samples = samples_per_channel * self.num_channels;
        let pcm = &self.pcm[..samples];

        self.buf.clear();
        self.buf.render_reserved(None);
        match self.num_channels {
            1 => {
                self.buf.chan_mut(0).copy_from_slice(pcm);
            }
            2 => {
                let (l, r) = self.buf.chan_pair_mut(0, 1);
                for (i, j) in (0..samples).step_by(2).enumerate() {
                    l[i] = pcm[j];
                    r[i] = pcm[j + 1];
                }
            }
            _ => {}
        }
        Ok(self.buf.as_audio_buffer_ref())
    }

    fn finalize(&mut self) -> symphonia_core::codecs::FinalizeResult {
        FinalizeResult::default()
    }

    fn last_decoded(&self) -> AudioBufferRef<'_> {
        self.buf.as_audio_buffer_ref()
    }
}

fn map_to_channels(num_channels: usize) -> Option<Channels> {
    let channels = match num_channels {
        1 => Layout::Mono.into_channels(),
        2 => Layout::Stereo.into_channels(),
        _ => return None,
    };

    Some(channels)
}

fn audio_buffer(
    sample_rate: u32,
    samples_per_channel: u64,
    num_channels: usize,
) -> AudioBuffer<i16> {
    let channels = map_to_channels(num_channels).expect("invalid channels");
    let spec = SignalSpec::new(sample_rate, channels);
    AudioBuffer::new(samples_per_channel, spec)
}
