use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, DeriveInput, Ident, ItemMod};

/// Derive macro generating an impl of `rustsynth::map::IntoOwnedMap`.
///
/// # Example
/// ```
/// use rustsynth::IntoOwnedMap;
///
/// #[derive(IntoOwnedMap)]
/// struct MyStruct {
///    field1: i32,
///     field2: String,
/// }
/// let s = MyStruct { field1: 42, field2: "Hello".to_string() };
/// let map = s.into_owned_map();
/// assert_eq!(map.get::<i32>("field1").unwrap(), &42);
/// assert_eq!(map.get::<String>("field2").unwrap(), &"Hello".to_string());
/// ```
#[proc_macro_derive(IntoOwnedMap)]
pub fn into_owned_map_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the From implementation
    impl_map_macro(&ast)
}

fn impl_map_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let fields: Vec<Ident> = match &ast.data {
        syn::Data::Struct(ds) => match &ds.fields {
            syn::Fields::Named(named) => named
                .named
                .iter()
                .map(|x| x.ident.clone().unwrap())
                .collect(),
            _ => panic!("Must have named fields"),
        },
        _ => panic!("Must be a data struct"),
    };
    let gen = quote! {
        impl rustsynth::map::IntoOwnedMap for #name {
            fn into_owned_map<'elem>(self) -> rustsynth::map::OwnedMap<'elem> {
                let mut map = rustsynth::map::OwnedMap::new();
                #(
                    map.set(stringify!(#fields), &self.#fields).unwrap();
                )*
                map
            }
        }
    };
    gen.into()
}

/// Macro to define a VapourSynth plugin containing multiple filters
#[proc_macro_attribute]
pub fn vapoursynth_plugin(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemMod);

    match generate_vs_plugin(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Macro to define individual filters within a plugin
#[proc_macro_attribute]
pub fn vapoursynth_filter(arg: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match generate_vs_filter(input, arg) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn generate_vs_plugin(input: ItemMod) -> syn::Result<proc_macro2::TokenStream> {
    let items = if let Some((_, items)) = &input.content {
        items
    } else {
        return Err(syn::Error::new_spanned(&input, "Module must have content"));
    };

    let expanded = quote! {
            #( #items )*

            // Plugin entry point - registers all filters
            #[no_mangle]
            pub unsafe extern "C" fn VapourSynthPluginInit2(
                plugin: *mut rustsynth::ffi::VSPlugin,
                vspapi: *const rustsynth::ffi::VSPLUGINAPI,
            ) {
                let api = &*vspapi;

                // Configure the plugin
                let identifier = std::ffi::CString::new(ID).unwrap();
                let namespace = std::ffi::CString::new(NAMESPACE).unwrap();
                let name = std::ffi::CString::new(NAME).unwrap();
                let plugin_version = PLUGIN_VER;
                let api_version = API_VER;
                let flags = FLAGS;

                api.configPlugin.expect("configPlugin is null")(
                    identifier.as_ptr(),
                    namespace.as_ptr(),
                    name.as_ptr(),
                    plugin_version,
                    api_version,
                    flags,
                    plugin
                );
                // Register all filters in this plugin
                __register_filters(plugin, vspapi);
            }
    };

    Ok(expanded)
}

fn generate_vs_filter(
    input: DeriveInput,
    arg: TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;

    // Create a clean version of the input without the vapoursynth_filter attribute
    let mut clean_input = input.clone();
    clean_input
        .attrs
        .retain(|attr| !attr.path().is_ident("vapoursynth_filter"));

    // Extract lifetime parameters
    let lifetimes = &input.generics.params;
    let has_lifetime = !lifetimes.is_empty();

    // Create the struct type with lifetimes
    let struct_type = if has_lifetime {
        quote! { #struct_name<'_> }
    } else {
        quote! { #struct_name }
    };

    // Generate unique C function names based on struct name
    let create_name = format!("{}Create", struct_name);
    let getframe_name = format!("{}GetFrame", struct_name);
    let free_name = format!("{}Free", struct_name);

    let create_ident = syn::Ident::new(&create_name, struct_name.span());
    let getframe_ident = syn::Ident::new(&getframe_name, struct_name.span());
    let free_ident = syn::Ident::new(&free_name, struct_name.span());

    // Common function signature for both video and audio filters
    let function_signature = quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #create_ident(
            in_: *const rustsynth::ffi::VSMap,
            out: *mut rustsynth::ffi::VSMap,
            user_data: *mut std::os::raw::c_void,
            core: *mut rustsynth::ffi::VSCore,
            vsapi: *const rustsynth::ffi::VSAPI,
        )
    };

    let create = match arg.to_string().as_str() {
        "video" => {
            quote! {
                #function_signature {
                    rustsynth::init_api(vsapi);
                    let api = &*vsapi;
                    std::panic::catch_unwind(|| {
                        let core_ref = rustsynth::core::CoreRef::from_ptr(core);
                        let in_map = rustsynth::map::MapRef::from_ptr(in_);
                        // Create filter instance from arguments
                        match <#struct_type>::from_args(&in_map, &core_ref) {
                            Ok(filter_data) => {
                                let deps = filter_data.get_dependencies();
                                let deps_ffi: Vec<rustsynth::ffi::VSFilterDependency> = deps.iter()
                                    .map(|d| d.as_ffi())
                                    .collect();

                                // Get filter mode from const
                                let filter_mode = <#struct_type>::MODE;
                                let media_info = match filter_data.get_video_info() {
                                    Ok(ai) => ai,
                                    Err(error_msg) => {
                                        let error_cstr = std::ffi::CString::new(error_msg).unwrap_or_else(|_| {
                                            std::ffi::CString::new("Failed to get video info").unwrap()
                                        });
                                        api.mapSetError.unwrap()(out, error_cstr.as_ptr());
                                        return;
                                    }
                                };

                                // Allocate filter data on heap
                                let data_ptr = Box::into_raw(Box::new(filter_data)) as *mut std::os::raw::c_void;
                                let filter_name = std::ffi::CString::new(<#struct_type>::NAME).unwrap();

                                api.createVideoFilter.unwrap()(
                                    out,
                                    filter_name.as_ptr(),
                                    &media_info.as_ffi() as *const rustsynth::ffi::VSVideoInfo,
                                    Some(#getframe_ident),
                                    Some(#free_ident),
                                    filter_mode.as_ffi() as i32,
                                    deps_ffi.as_ptr(),
                                    deps_ffi.len() as i32,
                                    data_ptr,
                                    core,
                                );
                            },
                            Err(error_msg) => {
                                let error_cstr = std::ffi::CString::new(error_msg).unwrap_or_else(|_| {
                                    std::ffi::CString::new("Filter creation failed").unwrap()
                                });
                                api.mapSetError.unwrap()(out, error_cstr.as_ptr());
                            }
                        }
                    }).unwrap_or_else(|_| {
                        api.mapSetError.unwrap()(out, b"Filter creation panicked\0".as_ptr() as *const std::os::raw::c_char);
                    });
                }
            }
        }
        "audio" => {
            quote! {
                #function_signature {
                    rustsynth::init_api(vsapi);
                    let api = &*vsapi;
                    std::panic::catch_unwind(|| {
                        let core_ref = rustsynth::core::CoreRef::from_ptr(core);
                        let in_map = rustsynth::map::Map::from_ptr(in_);
                        // Create filter instance from arguments
                        match <#struct_type>::from_args(&in_map, &core_ref) {
                            Ok(filter_data) => {
                                let deps = filter_data.get_dependencies();
                                let deps_ffi: Vec<rustsynth::ffi::VSFilterDependency> = deps.iter()
                                    .map(|d| d.as_ffi())
                                    .collect();

                                // Get filter mode from const
                                let filter_mode = <#struct_type>::MODE;
                                let media_info = match filter_data.get_audio_info() {
                                    Ok(ai) => ai,
                                    Err(error_msg) => {
                                        let error_cstr = std::ffi::CString::new(error_msg).unwrap_or_else(|_| {
                                            std::ffi::CString::new("Failed to get audio info").unwrap()
                                        });
                                        api.mapSetError.unwrap()(out, error_cstr.as_ptr());
                                        return;
                                    }
                                };

                                // Allocate filter data on heap
                                let data_ptr = Box::into_raw(Box::new(filter_data)) as *mut std::os::raw::c_void;
                                let filter_name = std::ffi::CString::new(<#struct_type>::NAME).unwrap();

                                api.createAudioFilter.unwrap()(
                                    out,
                                    filter_name.as_ptr(),
                                    &media_info,
                                    Some(#getframe_ident),
                                    Some(#free_ident),
                                    filter_mode.as_ffi(),
                                    deps_ffi.as_ptr(),
                                    deps_ffi.len() as i32,
                                    data_ptr,
                                    core,
                                );
                            },
                            Err(error_msg) => {
                                let error_cstr = std::ffi::CString::new(error_msg).unwrap_or_else(|_| {
                                    std::ffi::CString::new("Filter creation failed").unwrap()
                                });
                                api.mapSetError.unwrap()(out, error_cstr.as_ptr());
                            }
                        }
                    }).unwrap_or_else(|_| {
                        api.mapSetError.unwrap()(out, b"Filter creation panicked\0".as_ptr() as *const std::os::raw::c_char);
                    });
                }
            }
        }
        _ => {
            return Err(syn::Error::new_spanned(
                arg.to_string(),
                "Unsupported filter type. Use 'video' or 'audio'",
            ))
        }
    };

    let expanded = quote! {
        // Original struct definition
        #input

        #create

        // Frame processing function
        #[no_mangle]
        pub unsafe extern "C" fn #getframe_ident(
            n: i32,
            activation_reason: i32,
            instance_data: *mut std::os::raw::c_void,
            frame_data: *mut *mut std::os::raw::c_void,
            frame_ctx: *mut rustsynth::ffi::VSFrameContext,
            core: *mut rustsynth::ffi::VSCore,
            vsapi: *const rustsynth::ffi::VSAPI,
        ) -> *const rustsynth::ffi::VSFrame {
            let api = &*vsapi;

            std::panic::catch_unwind(|| {
                let filter = &mut *(instance_data as *mut #struct_type);
                let core_ref = rustsynth::core::CoreRef::from_ptr(core);
                let frame_ctx_wrapper = rustsynth::frame::FrameContext::from_ptr(frame_ctx);
                let activation = rustsynth::filter::ActivationReason::from_ffi(activation_reason);

                match activation {
                    rustsynth::filter::ActivationReason::Initial => {
                        // Request the frames we need
                        filter.request_input_frames(n, &frame_ctx_wrapper);
                        std::ptr::null()
                    },
                    rustsynth::filter::ActivationReason::AllFramesReady => {
                        // All frames ready - do the processing
                        // Convert frame_data to the expected format
                        let frame_data_array: &[u8; 4] = if (*frame_data).is_null() {
                            &[0; 4]
                        } else {
                            std::slice::from_raw_parts(*frame_data as *const u8, 4).try_into().unwrap_or(&[0; 4])
                        };

                        match filter.process_frame(n, frame_data_array, &frame_ctx_wrapper, core_ref) {
                            Ok(output_frame) => {
                                output_frame.as_ptr()
                            },
                            Err(error_msg) => {
                                let error_cstr = std::ffi::CString::new(error_msg).unwrap_or_else(|_| {
                                    std::ffi::CString::new("Frame processing failed").unwrap()
                                });
                                api.setFilterError.unwrap()(error_cstr.as_ptr(), frame_ctx);

                                // Clean up frame data if needed
                                if !(*frame_data).is_null() {
                                    filter.cleanup_frame_data(frame_data_array);
                                    *frame_data = std::ptr::null_mut();
                                }
                                std::ptr::null()
                            }
                        }
                    },
                    rustsynth::filter::ActivationReason::Error => {
                        // Error occurred - clean up
                        if !(*frame_data).is_null() {
                            let frame_data_array: &[u8; 4] = std::slice::from_raw_parts(*frame_data as *const u8, 4).try_into().unwrap_or(&[0; 4]);
                            filter.cleanup_frame_data(frame_data_array);
                            *frame_data = std::ptr::null_mut();
                        }
                        std::ptr::null()
                    }
                }
            }).unwrap_or_else(|_| {
                api.setFilterError.unwrap()(
                    b"Frame processing panicked\0".as_ptr() as *const std::os::raw::c_char,
                    frame_ctx
                );

                if !(*frame_data).is_null() {
                    *frame_data = std::ptr::null_mut();
                }
                std::ptr::null()
            })
        }

        // Filter cleanup function
        #[no_mangle]
        pub unsafe extern "C" fn #free_ident(
            instance_data: *mut std::os::raw::c_void,
            core: *mut rustsynth::ffi::VSCore,
            vsapi: *const rustsynth::ffi::VSAPI,
        ) {
            if !instance_data.is_null() {
                let _ = std::panic::catch_unwind(|| {
                    let filter = Box::from_raw(instance_data as *mut #struct_type);
                    filter.cleanup();
                    // Box drop handles memory cleanup
                });
            }
        }

        // Register this filter in the plugin
        impl<#lifetimes> #struct_name<#lifetimes> {
            fn register_filter(
                plugin: *mut rustsynth::ffi::VSPlugin,
                vspapi: *const rustsynth::ffi::VSPLUGINAPI
            ) {
                unsafe {
                    let api = &*vspapi;
                    let filter_name = std::ffi::CString::new(Self::NAME).unwrap();
                    let args_spec = std::ffi::CString::new(Self::ARGS).unwrap();
                    let return_spec = std::ffi::CString::new(Self::RETURNTYPE).unwrap();

                    if let Some(register_fn) = api.registerFunction {
                        let ret = register_fn(
                            filter_name.as_ptr(),
                            args_spec.as_ptr(),
                            return_spec.as_ptr(),
                            Some(#create_ident),
                            std::ptr::null_mut(),
                            plugin
                        );
                        if ret == 0 {
                            eprintln!("Failed to register filter '{}'", Self::NAME);
                        }
                    } else {
                        eprintln!("registerFunction API is NULL - cannot register filter '{}'", Self::NAME);
                    }
                }
            }
        }
    };

    Ok(expanded)
}
