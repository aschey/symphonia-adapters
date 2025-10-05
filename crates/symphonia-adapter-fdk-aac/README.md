# symphonia-adapter-fdk-aac

[![crates.io](https://img.shields.io/crates/v/symphonia-adapter-fdk-aac?logo=rust)](https://crates.io/crates/symphonia-adapter-fdk-aac)
[![docs.rs](https://img.shields.io/docsrs/symphonia-adapter-fdk-aac?logo=rust)](https://docs.rs/symphonia-adapter-fdk-aac)
![license](https://img.shields.io/badge/License-MIT%20or%20Apache%202-green.svg)
[![CI](https://github.com/aschey/symphonia-adapters/actions/workflows/ci.yml/badge.svg)](https://github.com/aschey/symphonia-adapters/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/aschey/symphonia-adapters/branch/main/graph/badge.svg?token=pF3FhV8OUt)](https://app.codecov.io/gh/aschey/symphonia-adapters)
![GitHub repo size](https://img.shields.io/github/repo-size/aschey/symphonia-adapters)
![Lines of Code](https://aschey.tech/tokei/github/aschey/symphonia-adapters)

Adapter for using [Fraunhofer FDK AAC](https://github.com/haileys/fdk-aac-rs)
with Symphonia. FDK AAC is a robust encoder/decoder for the AAC format.
Symphonia does have native AAC support, but it doesn't support the full spec.
Most notably,
[HE-AAC](https://en.wikipedia.org/wiki/High-Efficiency_Advanced_Audio_Coding)
support is currently missing.

## Usage

Ensure Symphonia's native AAC decoder is not also registered since they will
conflict with each other.

```rust
use symphonia_core::codecs::registry::CodecRegistry;
use symphonia_adapter_fdk_aac::AacDecoder;

let mut codec_registry = CodecRegistry::new();
codec_registry.register_audio_decoder::<AacDecoder>();
// register other codecs

// use codec_registry created above instead of symphonia::default::get_codecs();
```

## License

Original code in this crate is licensed under either the MIT or Apache-2.0
license, at your choice.

FDK AAC is licensed under
[a bespoke license](https://fedoraproject.org/wiki/Licensing/FDK-AAC).

Parts of this crate use modified code from other projects:

- Code adapted from [Symphonia](https://github.com/pdeljanov/Symphonia) is
  licensed under MPL-2.0.
- Code adapted from [Redlux](https://github.com/probablykasper/redlux) is
  licensed under MIT.

The original licenses have been preserved next to the relevant source files.
