# Introduction

RustSynth is a safe Rust wrapper for [VapourSynth](http://www.vapoursynth.com/), a powerful video processing framework. This book provides an in-depth exploration of how RustSynth works internally, how to write efficient filters, and how to integrate the library into different types of applications.

## What is RustSynth?

RustSynth is a fork of [vapoursynth-rs](https://github.com/YaLTeR/vapoursynth-rs) designed to support the latest VapourSynth API versions while maintaining safety and performance. The library provides:

- **Safe abstractions** over VapourSynth's C API
- **Zero-cost wrappers** that maintain performance
- **Modern procedural macros** for filter development
- **Comprehensive error handling** with descriptive messages
- **Memory safety** through Rust's ownership system

## Why This Book?

Unlike typical user documentation, this book focuses on understanding the library's internals and architecture. You'll learn:

- How RustSynth translates VapourSynth concepts into safe Rust
- The filter development lifecycle and performance implications
- Integration patterns for different application types
- Advanced techniques for building high-performance video processing pipelines

## Who Should Read This?

This book is intended for:

- **Filter developers** who want to write VapourSynth plugins in Rust
- **Application developers** integrating video processing capabilities
- **Contributors** to the RustSynth project
- **Advanced users** wanting to understand VapourSynth's Rust ecosystem

## Prerequisites

Readers should have:

- Solid understanding of Rust ownership, lifetimes, and unsafe code
- Basic knowledge of video processing concepts
- Familiarity with C FFI in Rust (helpful but not required)

## Book Structure

This book is organized into parts:

1. **Core Concepts** - Understanding RustSynth's architecture and design
2. **Filter Development** - Building video and audio processing filters
3. **Integration and Tools** - Working with RSPipe and VPY Scripts
4. **Advanced Topics** - Performance optimization and debugging

Each chapter includes detailed examples and links to the [API documentation](https://docs.rs/rustsynth) and [VapourSynth documentation](http://www.vapoursynth.com/doc/).

## Getting Started

To follow along with the examples, you'll need:

```toml
[dependencies]
rustsynth = "0.5"
```

For plugin development, also add:

```toml
[lib]
crate-type = ["cdylib"]
```
