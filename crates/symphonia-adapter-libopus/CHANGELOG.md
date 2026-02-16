# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.6](https://github.com/aschey/symphonia-adapters/compare/symphonia-adapter-libopus-v0.2.5..symphonia-adapter-libopus-v0.2.6) - 2026-02-16

### Bug Fixes

- Return error on invalid slice length instead of panicking ([#32](https://github.com/aschey/symphonia-adapters/issues/32)) - ([563ebcd](https://github.com/aschey/symphonia-adapters/commit/563ebcdd4bbde5988fb3dc3da100bb8045e9db23))


## [0.2.5](https://github.com/aschey/symphonia-adapters/compare/symphonia-adapter-libopus-v0.2.4..symphonia-adapter-libopus-v0.2.5) - 2025-12-20

### Bug Fixes

- Correct theoretical MAX_SAMPLES_PER_CHANNEL according to the Opus spec. ([#27](https://github.com/aschey/symphonia-adapters/issues/27)) - ([821e62f](https://github.com/aschey/symphonia-adapters/commit/821e62fe4859766096379bee5de346982ffb8aa7))

### Features

- Use f32 internally instead of i16. ([#26](https://github.com/aschey/symphonia-adapters/issues/26)) - ([ec15a10](https://github.com/aschey/symphonia-adapters/commit/ec15a10c9cb90e18f47918c6150b1b334e13ca46))
- Handle Opus error codes ([#24](https://github.com/aschey/symphonia-adapters/issues/24)) - ([85cf74d](https://github.com/aschey/symphonia-adapters/commit/85cf74d0edb06df882790fb56a35ad68ce04b4cb))


## [0.2.4](https://github.com/aschey/symphonia-adapters/compare/symphonia-adapter-libopus-v0.2.3..symphonia-adapter-libopus-v0.2.4) - 2025-12-18

### Bug Fixes

- Convert pre_skip to target sample_rate. ([#22](https://github.com/aschey/symphonia-adapters/issues/22)) - ([a68e560](https://github.com/aschey/symphonia-adapters/commit/a68e5600f8259f9df10933e79c0731136a7dbf91))


## [0.2.3](https://github.com/aschey/symphonia-adapters/compare/symphonia-adapter-libopus-v0.2.2..symphonia-adapter-libopus-v0.2.3) - 2025-10-13

### Features

- Add missing debug/clone implementations ([#14](https://github.com/aschey/symphonia-adapters/issues/14)) - ([9b0beb1](https://github.com/aschey/symphonia-adapters/commit/9b0beb1be02ada4e99e26810a191ea3790fa0a99))


## [0.2.2](https://github.com/aschey/symphonia-adapters/compare/symphonia-adapter-libopus-v0.2.1..symphonia-adapter-libopus-v0.2.2) - 2025-10-13

### Features

- Trim pre-skip samples ([#12](https://github.com/aschey/symphonia-adapters/issues/12)) - ([bb9f454](https://github.com/aschey/symphonia-adapters/commit/bb9f454b81e1dbcae2e73d38904e94e7f973b780))


## [0.2.1](https://github.com/aschey/symphonia-adapters/compare/symphonia-adapter-libopus-v0.2.0..symphonia-adapter-libopus-v0.2.1) - 2025-10-12

### Features

- Handle decoder reset ([#10](https://github.com/aschey/symphonia-adapters/issues/10)) - ([75bec26](https://github.com/aschey/symphonia-adapters/commit/75bec26971403a33ff7773c0da7cc07fb8a7d2d2))


## [0.2.0](https://github.com/aschey/symphonia-adapters/compare/symphonia-adapter-libopus-v0.1.2..symphonia-adapter-libopus-v0.2.0) - 2025-10-11

### Features

- Update `opusic-sys` and expose bundling options ([#8](https://github.com/aschey/symphonia-adapters/issues/8)) - ([4725f48](https://github.com/aschey/symphonia-adapters/commit/4725f4896e795d9ec414b6e8e45daf056da83749))


## [0.1.2](https://github.com/aschey/symphonia-adapters/compare/symphonia-adapter-libopus-v0.1.1..symphonia-adapter-libopus-v0.1.2) - 2025-10-06

### Features

- Use gapless info from demuxer ([#5](https://github.com/aschey/symphonia-adapters/issues/5)) - ([5837fe7](https://github.com/aschey/symphonia-adapters/commit/5837fe7c5e54a1f2d660f840f3e7517919eee801))


## [0.1.1](https://github.com/aschey/symphonia-adapters/compare/symphonia-adapter-libopus-v0.1.0..symphonia-adapter-libopus-v0.1.1) - 2025-10-05

### Documentation

- Add badges ([#3](https://github.com/aschey/symphonia-adapters/issues/3)) - ([0472c3a](https://github.com/aschey/symphonia-adapters/commit/0472c3a0571fcac8eccccfc659a0e1d605854bf4))


## [0.1.0](https://github.com/aschey/symphonia-adapters/releases/tag/symphonia-adapter-libopus-v0.1.0) - 2025-10-05
