# symphonia-adapter-libopus

Adapter for using [libopus](https://github.com/DoumanAsh/opusic-sys) with
Symphonia. Symphonia currently does not have native Opus support, so this crate
can provide it until a first-party solution is available.

See the [libopus binding documentation](https://crates.io/crates/opusic-sys) for
details on how to configure linking libopus.

## Usage

```rust
use symphonia_core::codecs::CodecRegistry;
use symphonia_adapter_libopus::OpusDecoder;

let mut codec_registry = CodecRegistry::new();
codec_registry.register_all::<OpusDecoder>();
// register other codecs

// use codec_registry created above instead of symphonia::default::get_codecs();
```

## License

This crate is licensed under either the MIT and Apache 2.0 license, at your
choice.

libopus and opusic-sys are licensed under the
[opus license](https://opus-codec.org/license/).
