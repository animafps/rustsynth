# RustSynth

[![crates.io](https://img.shields.io/crates/v/rustsynth.svg)](https://crates.io/crates/rustsynth)
[![docs.rs](https://docs.rs/rustsynth/badge.svg)](https://docs.rs/rustsynth)
[![License: LGPL v2.1](https://img.shields.io/badge/License-LGPL%20v2.1-blue.svg)](https://www.gnu.org/licenses/lgpl-2.1)

Safe & performant Rust bindings for VapourSynth video processing framework.

A modern fork of [vapoursynth-rs](https://github.com/YaLTeR/vapoursynth-rs) with support for the latest VapourSynth API versions.

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
rustsynth = "0.5"
```

## Documentation

- [API Documentation](https://docs.rs/rustsynth)
- [Development Documentation](https://animafps.github.io/rustsynth/rustsynth/index.html)

## Packages

This workspace contains three main packages:

### `rustsynth`

[![crates.io](https://img.shields.io/crates/v/rustsynth.svg)](https://crates.io/crates/rustsynth)

High-level safe wrapper for VapourSynth with a Rust-friendly API.

### `rustsynth-sys`

[![crates.io](https://img.shields.io/crates/v/rustsynth-sys.svg)](https://crates.io/crates/rustsynth-sys)

Low-level FFI bindings to VapourSynth C API.

### `rustsynth-derive`

[![crates.io](https://img.shields.io/crates/v/rustsynth-derive.svg)](https://crates.io/crates/rustsynth-derive)

Procedural macros for creating VapourSynth plugins.

## Community

Join our [Discord server](https://discord.com/invite/5z3YhWstQr) for support and development discussions.

## License

Licensed under LGPL-2.1. See [LICENSE](./LICENSE) for details.
