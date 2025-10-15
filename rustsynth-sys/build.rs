use std::env;
use std::path::PathBuf;

const LIBRARY_DIR_VARIABLE: &str = "VAPOURSYNTH_LIB_DIR";

fn main() {
    // Make sure the build script is re-run if our env variable is changed.
    println!("cargo:rerun-if-env-changed={}", LIBRARY_DIR_VARIABLE);

    let windows = env::var("TARGET").unwrap().contains("windows");

    // Get the default library dir on Windows.
    let default_library_dir = if windows {
        get_default_library_dir()
    } else {
        None
    };

    // Library directory override or the default dir on windows.
    if let Ok(dir) = env::var(LIBRARY_DIR_VARIABLE) {
        println!("cargo:rustc-link-search=native={}", dir);
    } else if let Some(default_library_dir) = default_library_dir {
        for dir in default_library_dir {
            println!("cargo:rustc-link-search=native={}", dir);
        }
    }

    if env::var("CARGO_FEATURE_VAPOURSYNTH_FUNCTIONS").is_ok() {
        println!("cargo:rustc-link-lib=vapoursynth");
    }

    if env::var("CARGO_FEATURE_VSSCRIPT_FUNCTIONS").is_ok() {
        let vsscript_lib_name = if windows {
            "vsscript"
        } else {
            "vapoursynth-script"
        };
        println!("cargo:rustc-link-lib={}", vsscript_lib_name);
    }

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let mut builder = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h");

    if env::var("CARGO_FEATURE_API_41").is_ok() {
        builder = builder.clang_arg("-DVS_USE_API_41");
    }

    if env::var("CARGO_FEATURE_VS_GRAPH_API").is_ok() {
        builder = builder.clang_arg("-DVS_GRAPH_API");
    }

    if env::var("CARGO_FEATURE_SCRIPT_API_42").is_ok() {
        builder = builder.clang_arg("-DVSSCRIPT_USE_API_42");
    }

    if env::var("CARGO_FEATURE_VSSCRIPT_FUNCTIONS").is_ok() {
        builder = builder
            .allowlist_function("getVSScriptAPI")
            .allowlist_var("VSSCRIPT.*");
    }

    let bindings = builder
        // Only include VapourSynth/VSScript items
        .allowlist_type("VS.*")
        .allowlist_function("vs.*")
        .allowlist_function("getVapourSynthAPI")
        .allowlist_var("VAPOURSYNTH.*")
        .allowlist_var("VS.*")
        .allowlist_var("VSH_.*")
        .allowlist_var("vs.*")
        .allowlist_var("pf.*") // VSPresetVideoFormat constants
        // https://github.com/rust-lang/rust-bindgen/issues/550
        .blocklist_type("max_align_t")
        .blocklist_function("_.*")
        // Block duplicate floating-point classification constants
        .blocklist_item("FP_NAN")
        .blocklist_item("FP_INFINITE")
        .blocklist_item("FP_ZERO")
        .blocklist_item("FP_SUBNORMAL")
        .blocklist_item("FP_NORMAL")
        // Block problematic 128-bit types from system headers
        .blocklist_type("__uint128_t")
        .blocklist_type("__int128_t")
        .blocklist_type("__int128")
        .blocklist_type("__uint128")
        // Block long double math functions (not used by VapourSynth)
        .blocklist_function(".*l$")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .rustified_enum("VSPropertyType")
        .rustified_enum("VSColorFamily")
        .rustified_enum("VSDataTypeHint")
        .rustified_enum("VSMapAppendMode")
        .rustified_enum("VSCacheMode")
        .rustified_enum("VSColorPrimaries")
        .rustified_enum("VSAudioChannels")
        .rustified_enum("VSActivationReason")
        .rustified_enum("VSColorRange")
        .rustified_enum("VSFilterMode")
        .rustified_enum("VSChromaLocation")
        .rustified_enum("VSFieldBased")
        .rustified_enum("VSMatrixCoefficients")
        .rustified_enum("VSMediaType")
        .rustified_enum("VSMessageType")
        .rustified_enum("VSSampleType")
        .rustified_enum("VSPresetFormat")
        .rustified_enum("VSPresetVideoFormat")
        .rustified_enum("VSMapPropertyError")
        .rustified_enum("VSRequestPattern")
        .rustified_enum("VSTransferCharacteristics")
        .bitfield_enum("VSCoreCreationFlags")
        .bitfield_enum("VSPluginConfigFlags")
        .translate_enum_integer_types(true)
        .use_core()
        .prepend_enum_name(false)
        .derive_eq(false)
        .derive_default(true) // Add Default derives where possible
        .derive_debug(true) // Add Debug derives
        .derive_copy(true) // Add Copy derives where safe
        .derive_hash(true) // Add Hash derives
        .formatter(bindgen::Formatter::Rustfmt)
        .size_t_is_usize(true)
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

// Returns the default library dirs on Windows.
// The default dir is where the VapourSynth installer puts the libraries.
fn get_default_library_dir() -> Option<impl Iterator<Item = String>> {
    let host = env::var("HOST").ok()?;

    // If the host isn't Windows we don't have %programfiles%.
    if !host.contains("windows") {
        return None;
    }

    let programfiles = env::var("programfiles").into_iter();

    // Add Program Files from the other bitness. This would be Program Files (x86) with a 64-bit
    // host and regular Program Files with a 32-bit host running on a 64-bit system.
    let programfiles = programfiles.chain(env::var(if host.starts_with("i686") {
        "programw6432"
    } else {
        "programfiles(x86)"
    }));

    let suffix = if env::var("TARGET").ok()?.starts_with("i686") {
        "lib32"
    } else {
        "lib64"
    };

    Some(programfiles.flat_map(move |programfiles| {
        // Use both VapourSynth and VapourSynth-32 folder names.
        ["", "-32"].iter().filter_map(move |vapoursynth_suffix| {
            let mut path = PathBuf::from(&programfiles);
            path.push(format!("VapourSynth{}", vapoursynth_suffix));
            path.push("sdk");
            path.push(suffix);
            path.to_str().map(|s| s.to_owned())
        })
    }))
}
