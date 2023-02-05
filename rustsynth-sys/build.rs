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

    #[cfg(feature = "vapoursynth-functions")]
    println!("cargo:rustc-link-lib=vapoursynth");

    let vsscript_lib_name = if windows {
        "vsscript"
    } else {
        "vapoursynth-script"
    };

    #[cfg(feature = "vsscript-functions")]
    println!("cargo:rustc-link-lib={}", vsscript_lib_name);

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // https://github.com/rust-lang/rust-bindgen/issues/550
        .blocklist_type("max_align_t")
        .blocklist_function("_.*")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .rustified_enum("*")
        .prepend_enum_name(false)
        .derive_eq(true)
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
