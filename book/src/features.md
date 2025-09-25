# Feature Flags

This document describes the available feature flags for the `rustsynth` crate. Feature flags allow you to enable optional functionality and control which parts of the VapourSynth API are available.

## Data Type Features

### `f16-pixel-type`

Enables support for 16-bit floating-point pixel components using the `half::f16` type from the `half` crate.

When enabled, allows working with video formats that use 2-byte floating-point samples. This is useful for high-precision video processing workflows.

**Dependencies:** `half` crate

### `proc-macro`

Enables procedural macro support by including the `rustsynth-derive` crate.

This feature provides derive macros and other procedural macros to simplify filter development and reduce boilerplate code.

**Dependencies:** `rustsynth-derive` crate

## Linking Features

These features control which VapourSynth libraries are linked at build time.

### `vapoursynth-functions`

Links to the main VapourSynth functions library. This is required for most core VapourSynth functionality including creating cores, working with nodes, and processing frames.

**Default:** Enabled

### `vsscript-functions`

Links to the VSScript functions library, which enables script evaluation and higher-level VapourSynth scripting functionality.

When enabled, the `vsscript` module becomes available with functions for evaluating VapourSynth scripts.

**Default:** Enabled

## API Version Features

These features control which versions of the VapourSynth API are available.

### `api-41`

Enables VapourSynth API version 4.1 headers and functionality.

Provides access to:

- [`CoreRef::clear_cache()`](https://docs.rs/rustsynth/latest/rustsynth/core/struct.CoreRef.html#method.clear_cache) - Clears all caches associated with the core
- [`Node::clear_cache()`](https://docs.rs/rustsynth/latest/rustsynth/node/struct.Node.html#method.clear_cache) - Clears all cached frames for a specific node
- [`RequestPattern::FrameReuseLastOnly`](https://docs.rs/rustsynth/latest/rustsynth/filter/enum.RequestPattern.html#variant.FrameReuseLastOnly) - Advanced frame reuse pattern for filters

**Default:** Enabled

### `script-api-42`

Enables VSScript API version 4.2 headers and functionality.

Provides access to:

- [`ScriptAPI::get_available_output_nodes()`](https://docs.rs/rustsynth/latest/rustsynth/vsscript/struct.ScriptAPI.html#method.get_available_output_nodes) - Get available output nodes from a script
- Enhanced VSScript environment functionality for listing output indices

**Default:** Enabled

### `graph-api`

Enables the experimental VapourSynth Graph API.

This is an unstable/experimental feature that provides access to:

- [`Node::get_creation_function_name()`](https://docs.rs/rustsynth/latest/rustsynth/node/struct.Node.html#method.get_creation_function_name) - Retrieve the function name used to create a node
- Advanced debugging and introspection capabilities

**Note:** This API is experimental and may change or be removed in future VapourSynth versions.

## Default Features

The following features are enabled by default:

- `vapoursynth-functions`
- `vsscript-functions`
- `api-41`
- `script-api-42`

## Usage

To use specific features in your `Cargo.toml`:

```toml
# Enable only core functionality
rustsynth = { version = "0.6", default-features = false, features = ["vapoursynth-functions"] }

# Enable all features including experimental ones
rustsynth = { version = "0.6", features = ["f16-pixel-type", "proc-macro", "graph-api"] }

# Custom feature combination
rustsynth = { version = "0.6", default-features = false, features = [
    "vapoursynth-functions",
    "vsscript-functions",
    "api-41",
    "f16-pixel-type"
] }
```
