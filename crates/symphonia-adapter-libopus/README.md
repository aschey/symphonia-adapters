# symphonia-adapter-libopus

Adapter for using [libopus](https://github.com/DoumanAsh/opusic-sys) with
Symphonia. Symphonia currently does not have native Opus support, so this crate
can provide it until a first-party solution is available.

## Usage

```rust
use symphonia_core::codecs::CodecRegistry;
use symphonia_adapter_libopus::OpusDecoder;

let mut codec_registry = CodecRegistry::new();
codec_registry.register_all::<OpusDecoder>();
```
