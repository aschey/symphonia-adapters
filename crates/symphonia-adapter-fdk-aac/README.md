# symphonia-adapter-fdk-aac

Adapter for using [fdk-aac](https://github.com/haileys/fdk-aac-rs) with
Symphonia. Symphonia has native AAC support, but it doesn't support the full
spec. Most notably,
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
```
