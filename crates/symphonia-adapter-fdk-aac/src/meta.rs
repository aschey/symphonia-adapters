use symphonia_core::audio::{Channels, Layout};
use symphonia_core::errors::Result;
use symphonia_core::io::{BitReaderLtr, ReadBitsLtr};

use crate::macros::validate;

#[derive(Default)]
pub(crate) struct M4AInfo {
    pub(crate) otype: M4AType,
    pub(crate) sample_rate: u32,
    pub(crate) sample_rate_index: u8,
    pub(crate) channels: u8,
    pub(crate) samples: usize,
}

impl M4AInfo {
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

    pub(crate) fn read(&mut self, buf: &[u8]) -> Result<()> {
        let mut bs = BitReaderLtr::new(buf);

        self.otype = Self::read_object_type(&mut bs)?;
        self.sample_rate = Self::read_sampling_frequency(&mut bs)?;
        self.sample_rate_index = sample_rate_index(self.sample_rate);

        validate!(self.sample_rate > 0);

        self.channels = Self::read_channel_config(&mut bs)? as u8;

        if (self.otype == M4AType::Sbr) || (self.otype == M4AType::PS) {
            let _ext_srate = Self::read_sampling_frequency(&mut bs)?;
            self.otype = Self::read_object_type(&mut bs)?;

            let _ext_chans = if self.otype == M4AType::ER_BSAC {
                Self::read_channel_config(&mut bs)?
            } else {
                0
            };
        }
        let short_frame = bs.read_bool()?;
        self.samples = if short_frame { 960 } else { 1024 };

        Ok(())
    }
}

impl std::fmt::Display for M4AInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MPEG 4 Audio {}, {} Hz, {} channels, {} samples per frame",
            self.otype, self.sample_rate, self.channels, self.samples
        )
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Default, Copy, Debug, PartialEq, Eq)]
pub enum M4AType {
    #[default]
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
    M4AType::Reserved, /* escape */
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

const AAC_SAMPLE_RATES: [u32; 16] = [
    96000, 88200, 64000, 48000, 44100, 32000, 24000, 22050, 16000, 12000, 11025, 8000, 7350, 0, 0,
    0,
];

pub(crate) fn sample_rate_index(sample_rate: u32) -> u8 {
    AAC_SAMPLE_RATES
        .iter()
        .position(|s| *s == sample_rate)
        .unwrap_or_default() as u8
}

const AAC_CHANNELS: [usize; 8] = [0, 1, 2, 3, 4, 5, 6, 8];

pub(crate) fn map_to_channels(num_channels: u8) -> Option<Channels> {
    let channels = match num_channels {
        1 => Layout::Mono.into_channels(),
        2 => Layout::Stereo.into_channels(),
        _ => return None,
    };

    Some(channels)
}
