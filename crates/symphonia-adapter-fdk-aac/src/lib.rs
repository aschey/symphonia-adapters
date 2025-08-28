use std::ops::Range;

use fdk_aac::dec::{Decoder, Transport};
use symphonia_core::{
    audio::{
        AsGenericAudioBufferRef, AudioBuffer, AudioMut, AudioSpec, Channels, GenericAudioBufferRef,
        layouts,
    },
    codec_profile,
    codecs::{
        CodecInfo,
        audio::{
            AudioCodecParameters, AudioDecoder, AudioDecoderOptions, FinalizeResult,
            well_known::{
                CODEC_ID_AAC,
                profiles::{CODEC_PROFILE_AAC_LTP, CODEC_PROFILE_AAC_SSR},
            },
        },
        registry::{RegisterableAudioDecoder, SupportedAudioCodec},
    },
    formats::Packet,
    io::{BitReaderLtr, FiniteBitStream, ReadBitsLtr},
    support_audio_codec,
};

pub struct AacDecoder {
    decoder: Decoder,
    buf: AudioBuffer<i16>,
    codec_params: AudioCodecParameters,
    m4a_info: M4AInfo,
    m4a_info_validated: bool,
}

impl AacDecoder {
    pub fn new(
        params: &AudioCodecParameters,
        _opts: &AudioDecoderOptions,
    ) -> symphonia_core::errors::Result<Self> {
        println!("{params:?}");
        let mut m4a_info = M4AInfo::new();
        let mut m4a_info_validated = false;
        // If extra data present, parse the audio specific config
        if let Some(extra_data_buf) = &params.extra_data {
            // validate!(extra_data_buf.len() >= 2);
            m4a_info.read(extra_data_buf)?;
            m4a_info_validated = true;
        } else {
            // Otherwise, assume there is no ASC and use the codec parameters for ADTS.
            m4a_info.otype = M4AType::Lc;
            // m4a_info.samples = 1024;
            // m4ainfo.srate = 44100;

            m4a_info.srate = match params.sample_rate {
                Some(rate) => rate,
                None => 0, //return unsupported_error("aac: sample rate is required"),
            };

            m4a_info.channels = if let Some(channels) = &params.channels {
                channels.count()
            } else {
                0 //return unsupported_error("aac: channels or channel layout is required");
            };
        }
        let decoder = Decoder::new(Transport::Adts);
        println!("CHANNELS {}", m4a_info.channels);
        let channels = map_to_channels(m4a_info.channels).unwrap();

        let buf = AudioBuffer::new(AudioSpec::new(m4a_info.srate, channels), m4a_info.samples);
        Ok(Self {
            decoder,
            codec_params: params.clone(),
            buf,
            m4a_info,
            m4a_info_validated,
        })
    }
}

impl AudioDecoder for AacDecoder {
    fn reset(&mut self) {}

    fn codec_info(&self) -> &CodecInfo {
        &Self::supported_codecs().first().unwrap().info
    }

    fn codec_params(&self) -> &AudioCodecParameters {
        &self.codec_params
    }

    fn decode(
        &mut self,
        packet: &Packet,
    ) -> symphonia_core::errors::Result<GenericAudioBufferRef<'_>> {
        let adts_header = construct_adts_header(
            self.m4a_info.otype,
            AAC_SAMPLE_RATES
                .iter()
                .position(|s| *s == self.m4a_info.srate)
                .unwrap() as u8,
            self.m4a_info.channels as u8,
            packet.buf().len(),
        );
        let bytes_filled = self
            .decoder
            .fill(&[&adts_header, packet.buf()].concat())
            .unwrap();
        println!("filled {bytes_filled}");
        let mut pcm = vec![0; 8192];
        match self.decoder.decode_frame(&mut pcm) {
            Ok(_) => {
                println!("SUCCESS");
            }
            Err(e) => {
                println!("{e:?}");
                self.buf.clear();

                return Ok(self.buf.as_generic_audio_buffer_ref());
            }
        }
        if !self.m4a_info_validated {
            let stream_info = self.decoder.stream_info();
            let capacity = self.decoder.decoded_frame_size();
            let channels = stream_info.numChannels as usize;
            println!("{stream_info:?}");
            println!("{capacity}");
            self.m4a_info = M4AInfo {
                otype: M4A_TYPES[stream_info.aot as usize],
                channels: stream_info.numChannels as usize,
                srate: stream_info.aacSampleRate as u32,
                samples: capacity / channels,
                ps_present: false,
                sbr_present: false,
                sbr_ps_info: None,
            };
            let channels = map_to_channels(self.m4a_info.channels).unwrap();

            self.buf = AudioBuffer::new(
                AudioSpec::new(self.m4a_info.srate, channels),
                self.m4a_info.samples,
            );
            self.m4a_info_validated = true;
        }

        let num_channels = self.decoder.stream_info().numChannels;
        // let sample_rate = self.decoder.stream_info().sampleRate;
        let capacity = self.decoder.decoded_frame_size();
        //
        // let channels = map_to_channels(num_channels as usize).unwrap();
        // println!("{:?}", self.decoder.stream_info());
        // println!("capacity {capacity}");

        //  self.buf = AudioBuffer::new(AudioSpec::new(sample_rate as u32, channels), capacity);

        let pcm = &pcm[..capacity];
        self.buf.clear();

        self.buf.render_uninit(None);
        // match num_channels {
        //     1 => {
        //         self.buf.plane_mut(0).unwrap().copy_from_slice(pcm);
        //     }
        //     2 => {
        //         let (l, r) = self.buf.plane_pair_mut(0, 1).unwrap();
        //         for (j, i) in (0..pcm.len()).step_by(2).enumerate() {
        //             l[j] = pcm[i];
        //             r[j] = pcm[i + 1];
        //         }
        //     }
        //     _ => {
        //         unreachable!()
        //     }
        // }
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
        println!("HERE");
        Ok(Box::new(AacDecoder::new(params, opts)?))
    }

    fn supported_codecs() -> &'static [SupportedAudioCodec] {
        use symphonia_core::codecs::audio::well_known::profiles::CODEC_PROFILE_AAC_LC;

        &[support_audio_codec!(
            CODEC_ID_AAC,
            "aac",
            "Advanced Audio Coding",
            &[codec_profile!(
                CODEC_PROFILE_AAC_LC,
                "aac-lc",
                "Low Complexity"
            ),]
        )]
    }
}

struct M4AInfo {
    otype: M4AType,
    srate: u32,
    channels: usize,
    samples: usize,
    sbr_ps_info: Option<(u32, usize)>,
    sbr_present: bool,
    ps_present: bool,
}

impl M4AInfo {
    fn new() -> Self {
        Self {
            otype: M4AType::None,
            srate: 0,
            channels: 0,
            samples: 0,
            sbr_ps_info: Option::None,
            sbr_present: false,
            ps_present: false,
        }
    }

    fn read_object_type<B: ReadBitsLtr>(bs: &mut B) -> symphonia_core::errors::Result<M4AType> {
        let otypeidx = match bs.read_bits_leq32(5)? {
            idx if idx < 31 => idx as usize,
            31 => (bs.read_bits_leq32(6)? + 32) as usize,
            _ => unreachable!(),
        };

        if otypeidx >= M4A_TYPES.len() {
            Ok(M4AType::Unknown)
        } else {
            Ok(M4A_TYPES[otypeidx])
        }
    }

    fn read_sampling_frequency<B: ReadBitsLtr>(bs: &mut B) -> symphonia_core::errors::Result<u32> {
        match bs.read_bits_leq32(4)? {
            idx if idx < 15 => Ok(AAC_SAMPLE_RATES[idx as usize]),
            _ => {
                let srate = (0xf << 20) & bs.read_bits_leq32(20)?;
                Ok(srate)
            }
        }
    }

    fn read_channel_config<B: ReadBitsLtr>(bs: &mut B) -> symphonia_core::errors::Result<usize> {
        let chidx = bs.read_bits_leq32(4)? as usize;
        if chidx < AAC_CHANNELS.len() {
            Ok(AAC_CHANNELS[chidx])
        } else {
            Ok(chidx)
        }
    }

    fn read(&mut self, buf: &[u8]) -> symphonia_core::errors::Result<()> {
        let mut bs = BitReaderLtr::new(buf);

        self.otype = Self::read_object_type(&mut bs)?;
        self.srate = Self::read_sampling_frequency(&mut bs)?;

        //validate!(self.srate > 0);

        self.channels = Self::read_channel_config(&mut bs)?;
        println!("CHANNELS2 {}", self.channels);

        if (self.otype == M4AType::Sbr) || (self.otype == M4AType::PS) {
            let ext_srate = Self::read_sampling_frequency(&mut bs)?;
            self.otype = Self::read_object_type(&mut bs)?;

            let ext_chans = if self.otype == M4AType::ER_BSAC {
                Self::read_channel_config(&mut bs)?
            } else {
                0
            };

            self.sbr_ps_info = Some((ext_srate, ext_chans));
        }

        match self.otype {
            M4AType::Main
            | M4AType::Lc
            | M4AType::Ssr
            | M4AType::Scalable
            | M4AType::TwinVQ
            | M4AType::ER_AAC_LC
            | M4AType::ER_AAC_LTP
            | M4AType::ER_AAC_Scalable
            | M4AType::ER_TwinVQ
            | M4AType::ER_BSAC
            | M4AType::ER_AAC_LD => {
                // GASpecificConfig
                let short_frame = bs.read_bool()?;

                self.samples = if short_frame { 960 } else { 1024 };

                let depends_on_core = bs.read_bool()?;

                if depends_on_core {
                    let _delay = bs.read_bits_leq32(14)?;
                }

                let extension_flag = bs.read_bool()?;

                if self.channels == 0 {
                    // return unsupported_error("aac: program config element");
                }

                if (self.otype == M4AType::Scalable) || (self.otype == M4AType::ER_AAC_Scalable) {
                    let _layer = bs.read_bits_leq32(3)?;
                }

                if extension_flag {
                    if self.otype == M4AType::ER_BSAC {
                        let _num_subframes = bs.read_bits_leq32(5)? as usize;
                        let _layer_length = bs.read_bits_leq32(11)?;
                    }

                    if (self.otype == M4AType::ER_AAC_LC)
                        || (self.otype == M4AType::ER_AAC_LTP)
                        || (self.otype == M4AType::ER_AAC_Scalable)
                        || (self.otype == M4AType::ER_AAC_LD)
                    {
                        let _section_data_resilience = bs.read_bool()?;
                        let _scalefactors_resilience = bs.read_bool()?;
                        let _spectral_data_resilience = bs.read_bool()?;
                    }

                    let extension_flag3 = bs.read_bool()?;

                    if extension_flag3 {
                        // return unsupported_error("aac: version3 extensions");
                    }
                }
            }
            // M4AType::Celp => {
            //     return unsupported_error("aac: CELP config");
            // }
            // M4AType::Hvxc => {
            //     return unsupported_error("aac: HVXC config");
            // }
            // M4AType::Ttsi => {
            //     return unsupported_error("aac: TTS config");
            // }
            // M4AType::MainSynth
            // | M4AType::WavetableSynth
            // | M4AType::GeneralMIDI
            // | M4AType::Algorithmic => {
            //     return unsupported_error("aac: structured audio config");
            // }
            // M4AType::ER_CELP => {
            //     return unsupported_error("aac: ER CELP config");
            // }
            // M4AType::ER_HVXC => {
            //     return unsupported_error("aac: ER HVXC config");
            // }
            // M4AType::ER_HILN | M4AType::ER_Parametric => {
            //     return unsupported_error("aac: parametric config");
            // }
            // M4AType::Ssc => {
            //     return unsupported_error("aac: SSC config");
            // }
            // M4AType::MPEGSurround => {
            //     // bs.ignore_bits(1)?; // sacPayloadEmbedding
            //     return unsupported_error("aac: MPEG Surround config");
            // }
            // M4AType::Layer1 | M4AType::Layer2 | M4AType::Layer3 => {
            //     return unsupported_error("aac: MPEG Layer 1/2/3 config");
            // }
            // M4AType::Dst => {
            //     return unsupported_error("aac: DST config");
            // }
            // M4AType::Als => {
            //     // bs.ignore_bits(5)?; // fillBits
            //     return unsupported_error("aac: ALS config");
            // }
            // M4AType::Sls | M4AType::SLSNonCore => {
            //     return unsupported_error("aac: SLS config");
            // }
            // M4AType::ER_AAC_ELD => {
            //     return unsupported_error("aac: ELD config");
            // }
            // M4AType::SMRSimple | M4AType::SMRMain => {
            //     return unsupported_error("aac: symbolic music config");
            // }
            _ => {}
        };

        match self.otype {
            M4AType::ER_AAC_LC
            | M4AType::ER_AAC_LTP
            | M4AType::ER_AAC_Scalable
            | M4AType::ER_TwinVQ
            | M4AType::ER_BSAC
            | M4AType::ER_AAC_LD
            | M4AType::ER_CELP
            | M4AType::ER_HVXC
            | M4AType::ER_HILN
            | M4AType::ER_Parametric
            | M4AType::ER_AAC_ELD => {
                let ep_config = bs.read_bits_leq32(2)?;

                if (ep_config == 2) || (ep_config == 3) {
                    //  return unsupported_error("aac: error protection config");
                }
                if ep_config == 3 {
                    let direct_mapping = bs.read_bit()?;
                    //     validate!(direct_mapping);
                }
            }
            _ => {}
        };

        if self.sbr_ps_info.is_some() && (bs.bits_left() >= 16) {
            let sync = bs.read_bits_leq32(11)?;

            if sync == 0x2B7 {
                let ext_otype = Self::read_object_type(&mut bs)?;
                if ext_otype == M4AType::Sbr {
                    self.sbr_present = bs.read_bool()?;
                    if self.sbr_present {
                        let _ext_srate = Self::read_sampling_frequency(&mut bs)?;
                        if bs.bits_left() >= 12 {
                            let sync = bs.read_bits_leq32(11)?;
                            if sync == 0x548 {
                                self.ps_present = bs.read_bool()?;
                            }
                        }
                    }
                }
                if ext_otype == M4AType::PS {
                    self.sbr_present = bs.read_bool()?;
                    if self.sbr_present {
                        let _ext_srate = Self::read_sampling_frequency(&mut bs)?;
                    }
                    let _ext_channels = bs.read_bits_leq32(4)?;
                }
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for M4AInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MPEG 4 Audio {}, {} Hz, {} channels, {} samples per frame",
            self.otype, self.srate, self.channels, self.samples
        )
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum M4AType {
    None,
    Main,
    Lc,
    Ssr,
    Ltp,
    Sbr,
    Scalable,
    TwinVQ,
    Celp,
    Hvxc,
    Ttsi,
    MainSynth,
    WavetableSynth,
    GeneralMIDI,
    Algorithmic,
    ER_AAC_LC,
    ER_AAC_LTP,
    ER_AAC_Scalable,
    ER_TwinVQ,
    ER_BSAC,
    ER_AAC_LD,
    ER_CELP,
    ER_HVXC,
    ER_HILN,
    ER_Parametric,
    Ssc,
    PS,
    MPEGSurround,
    Layer1,
    Layer2,
    Layer3,
    Dst,
    Als,
    Sls,
    SLSNonCore,
    ER_AAC_ELD,
    SMRSimple,
    SMRMain,
    Reserved,
    Unknown,
}

pub const M4A_TYPES: &[M4AType] = &[
    M4AType::None,
    M4AType::Main,
    M4AType::Lc,
    M4AType::Ssr,
    M4AType::Ltp,
    M4AType::Sbr,
    M4AType::Scalable,
    M4AType::TwinVQ,
    M4AType::Celp,
    M4AType::Hvxc,
    M4AType::Reserved,
    M4AType::Reserved,
    M4AType::Ttsi,
    M4AType::MainSynth,
    M4AType::WavetableSynth,
    M4AType::GeneralMIDI,
    M4AType::Algorithmic,
    M4AType::ER_AAC_LC,
    M4AType::Reserved,
    M4AType::ER_AAC_LTP,
    M4AType::ER_AAC_Scalable,
    M4AType::ER_TwinVQ,
    M4AType::ER_BSAC,
    M4AType::ER_AAC_LD,
    M4AType::ER_CELP,
    M4AType::ER_HVXC,
    M4AType::ER_HILN,
    M4AType::ER_Parametric,
    M4AType::Ssc,
    M4AType::PS,
    M4AType::MPEGSurround,
    M4AType::Reserved, /*escape*/
    M4AType::Layer1,
    M4AType::Layer2,
    M4AType::Layer3,
    M4AType::Dst,
    M4AType::Als,
    M4AType::Sls,
    M4AType::SLSNonCore,
    M4AType::ER_AAC_ELD,
    M4AType::SMRSimple,
    M4AType::SMRMain,
];

impl std::fmt::Display for M4AType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", M4A_TYPE_NAMES[*self as usize])
    }
}
// https://en.wikipedia.org/wiki/MPEG-4_Part_3#MPEG-4_Audio_Object_Types
pub const M4A_TYPE_NAMES: &[&str] = &[
    "None",
    "AAC Main",
    "AAC LC",
    "AAC SSR",
    "AAC LTP",
    "SBR",
    "AAC Scalable",
    "TwinVQ",
    "CELP",
    "HVXC",
    // "(reserved10)",
    // "(reserved11)",
    "TTSI",
    "Main synthetic",
    "Wavetable synthesis",
    "General MIDI",
    "Algorithmic Synthesis and Audio FX",
    "ER AAC LC",
    // "(reserved18)",
    "ER AAC LTP",
    "ER AAC Scalable",
    "ER TwinVQ",
    "ER BSAC",
    "ER AAC LD",
    "ER CELP",
    "ER HVXC",
    "ER HILN",
    "ER Parametric",
    "SSC",
    "PS",
    "MPEG Surround",
    // "(escape)",
    "Layer-1",
    "Layer-2",
    "Layer-3",
    "DST",
    "ALS",
    "SLS",
    "SLS non-core",
    "ER AAC ELD",
    "SMR Simple",
    "SMR Main",
    "(reserved)",
    "(unknown)",
];

pub const AAC_SAMPLE_RATES: [u32; 16] = [
    96000, 88200, 64000, 48000, 44100, 32000, 24000, 22050, 16000, 12000, 11025, 8000, 7350, 0, 0,
    0,
];

/// Mapping of AAC channel configuration bits to number of channels.
pub const AAC_CHANNELS: [usize; 8] = [0, 1, 2, 3, 4, 5, 6, 8];

pub fn map_to_channels(num_channels: usize) -> Option<Channels> {
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

pub fn construct_adts_header(
    object_type: M4AType,
    sample_freq_index: u8,
    channel_config: u8,
    num_bytes: usize,
) -> Vec<u8> {
    // ADTS header wiki reference: https://wiki.multimedia.cx/index.php/ADTS#:~:text=Audio%20Data%20Transport%20Stream%20(ADTS,to%20stream%20audio%2C%20usually%20AAC.

    // byte7 and byte9 not included without CRC
    let adts_header_length = 7;

    // AAAA_AAAA
    let byte0 = 0b1111_1111;

    // AAAA_BCCD
    // D: Only support 1 (without CRC)
    let byte1 = 0b1111_0001;

    // EEFF_FFGH
    let mut byte2 = 0b0000_0000;
    // let object_type = match object_type {
    //     AudioObjectType::AacLowComplexity => 2,
    //     // Audio object types 5 (SBR) and 29 (PS) are coerced to type 2 (AAC-LC).
    //     // The decoder will have to detect SBR/PS. This is called "Implicit
    //     // Signaling" and it's the only option for ADTS.
    //     AudioObjectType::SpectralBandReplication => 2, // SBR, needed to support HE-AAC v1
    //     AudioObjectType::ParametricStereo => 2,        // PS, needed to support HE-AAC v2
    //     aot => return Err(Error::UnsupportedObjectType(aot)),
    // };
    let adts_object_type = object_type as u8 - 1;
    byte2 = (byte2 << 2) | adts_object_type; // EE

    // let sample_freq_index = match sample_freq_index {
    //     SampleFreqIndex::Freq96000 => 0,
    //     SampleFreqIndex::Freq88200 => 1,
    //     SampleFreqIndex::Freq64000 => 2,
    //     SampleFreqIndex::Freq48000 => 3,
    //     SampleFreqIndex::Freq44100 => 4,
    //     SampleFreqIndex::Freq32000 => 5,
    //     SampleFreqIndex::Freq24000 => 6,
    //     SampleFreqIndex::Freq22050 => 7,
    //     SampleFreqIndex::Freq16000 => 8,
    //     SampleFreqIndex::Freq12000 => 9,
    //     SampleFreqIndex::Freq11025 => 10,
    //     SampleFreqIndex::Freq8000 => 11,
    //     SampleFreqIndex::Freq7350 => 12,
    //     // 13-14 = reserved
    //     // 15 = explicit frequency (forbidden in adts)
    // };
    byte2 = (byte2 << 4) | sample_freq_index; // FFFF
    byte2 = (byte2 << 1) | 0b1; // G
    //
    // let channel_config = match channel_config {
    //     // 0 = for when channel config is sent via an inband PCE
    //     ChannelConfig::Mono => 1,
    //     ChannelConfig::Stereo => 2,
    //     ChannelConfig::Three => 3,
    //     ChannelConfig::Four => 4,
    //     ChannelConfig::Five => 5,
    //     ChannelConfig::FiveOne => 6,
    //     ChannelConfig::SevenOne => 7,
    //     // 8-15 = reserved
    // };
    byte2 = (byte2 << 1) | get_bits_u8(channel_config, 6..6); // H

    // HHIJ_KLMM
    let mut byte3 = 0b0000_0000;
    byte3 = (byte3 << 2) | get_bits_u8(channel_config, 7..8); // HH
    byte3 = (byte3 << 4) | 0b1111; // IJKL

    let frame_length = adts_header_length + num_bytes as u16;
    byte3 = (byte3 << 2) | get_bits(frame_length, 3..5) as u8; // MM

    // MMMM_MMMM
    let byte4 = get_bits(frame_length, 6..13) as u8;

    // MMMO_OOOO
    let mut byte5 = 0b0000_0000;
    byte5 = (byte5 << 3) | get_bits(frame_length, 14..16) as u8;
    byte5 = (byte5 << 5) | 0b11111; // OOOOO

    // OOOO_OOPP
    let mut byte6 = 0b0000_0000;
    byte6 = (byte6 << 6) | 0b111111; // OOOOOO
    byte6 = (byte6 << 2) | 0b00; // PP

    return vec![byte0, byte1, byte2, byte3, byte4, byte5, byte6];
}

fn get_bits(byte: u16, range: Range<u16>) -> u16 {
    let shaved_left = byte << range.start - 1;
    let moved_back = shaved_left >> range.start - 1;
    let shave_right = moved_back >> 16 - range.end;
    return shave_right;
}

fn get_bits_u8(byte: u8, range: Range<u8>) -> u8 {
    let shaved_left = byte << range.start - 1;
    let moved_back = shaved_left >> range.start - 1;
    let shave_right = moved_back >> 8 - range.end;
    return shave_right;
}

