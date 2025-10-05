# symphonia-adapter-fdk-aac

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
use symphonia_core::codecs::CodecRegistry;
use symphonia_adapter_fdk_aac::AacDecoder;

let mut codec_registry = CodecRegistry::new();
codec_registry.register_all::<AacDecoder>();
// register other codecs

// use codec_registry created above instead of symphonia::default::get_codecs();
```

## License

Original code in this crate is licensed under the MIT or Apache-2.0 licenses.

FDK AAC is licensed under
[a bespoke license](https://fedoraproject.org/wiki/Licensing/FDK-AAC).

Parts of this crate use modified code from other projects:

- Code adapted from [Symphonia](https://github.com/pdeljanov/Symphonia) is
  licensed under MPL-2.0.
- Code adapted from [Redlux](https://github.com/probablykasper/redlux) is
  licensed under MIT.

The original licenses have been preserved next to the relevant source files.
