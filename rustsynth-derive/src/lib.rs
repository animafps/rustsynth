use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use rustsynth::core::{CoreCreationFlags, CoreRef};
use syn::{self, parse_macro_input, DeriveInput, Ident, ItemMod};

/// Derive macro generating an impl of the trait `OwnedMap`.
#[proc_macro_derive(OwnedMap)]
pub fn owned_map_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
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
        impl OwnedMap for #name {
            fn to_map<'elem>(self) -> rustsynth::map::OwnedMap<'elem> {
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

/// Initilizes the autoloaded plugins
///
/// # Example
///
/// ```
/// use rustsynth_derive::init_plugins;
/// use rustsynth::{core::{CoreRef,CoreCreationFlags},plugin::Plugin};
///
/// let mycore = CoreRef::new(CoreCreationFlags::NONE);
/// init_plugins!();
///
/// let clip = Plugins::ffms2::Source(&mycore, "./demo.mp4".to_owned()).get_node("clip").unwrap();
/// ```
#[proc_macro]
pub fn init_plugins(_input: TokenStream) -> TokenStream {
    let core = CoreRef::new(CoreCreationFlags::NONE);
    let plugins = core.plugins();
    let token_vec: Vec<proc_macro2::TokenStream> = plugins
        .map(|x| {
            let namespace = Ident::new(&x.namespace().unwrap(), Span::call_site());
            let func_vec: Vec<proc_macro2::TokenStream> = x
                .functions()
                .map(|y| {
                    let name = syn::parse_str::<Ident>(y.name.unwrap()).unwrap_or_else(|_| syn::parse_str::<Ident>(&(y.name.unwrap().to_owned() + "_")).expect("error"));

                    let args = y
                        .arguments
                        .unwrap();
                    let args_split: Vec<Vec<&str>>  = args
                        .split(";")
                        .map(|z| z.split(":").collect::<Vec<&str>>())
                        .collect();
                    let args_vec = parse_arguments(&args_split);
                    let arg_names: Vec<Ident> = args_split.iter().filter(|x| x.len() == 2).map(|x| {
                        syn::parse_str::<Ident>(x[0]).unwrap_or_else(|_| {
                            syn::parse_str::<Ident>(&(x[0].to_owned() + "_")).expect("error")
                        })
                    }).collect();
                    quote! {
                        pub fn #name<'core>(core: &'core rustsynth::core::CoreRef<'core>, #(#args_vec),*) -> rustsynth::map::OwnedMap<'core> {
                            let p = core.plugin_by_namespace(stringify!(#namespace)).unwrap();
                            let mut in_args = rustsynth::map::OwnedMap::new();
                            #(
                                in_args.set(stringify!(#arg_names), &#arg_names).expect(("Cannot set ".to_owned() + stringify!(#arg_names)).as_str());
                            )*
                            p.invoke(stringify!(#name), &in_args)
                        }
                    }
                })
                .collect();
            quote! {
                pub mod #namespace {
                    #(
                        #func_vec
                    )*
                }
            }
        })
        .collect();
    let gen = quote! {
        #[allow(non_snake_case)]
        pub mod Plugins {
            #(
                #token_vec
            )*
        }
    };
    unsafe { core.free_core() };
    gen.into()
}

fn parse_arguments(input: &Vec<Vec<&str>>) -> Vec<proc_macro2::TokenStream> {
    input
        .iter()
        .filter(|x| x.len() == 2)
        .map(|x| {
            let x0 = syn::parse_str::<Ident>(x[0]).unwrap_or_else(|_| {
                syn::parse_str::<Ident>(&(x[0].to_owned() + "_")).expect("error")
            });
            match x[1] {
                "vnode" => {
                    quote! {
                        #x0: rustsynth::node::Node
                    }
                }
                "int" => {
                    quote! {
                        #x0: i64
                    }
                }
                "data" => {
                    quote! {
                        #x0: String
                    }
                }
                //y => {
                //    quote! {
                //        #x0: #y
                //    }
                //}
                _ => {
                    quote! {
                        #x0: i64
                    }
                }
            }
        })
        .collect()
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
    match generate_vs_filter(input,arg) {
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

fn generate_vs_filter(input: DeriveInput, arg: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;

    // Generate unique C function names based on struct name
    let create_name = format!("{}Create", struct_name);
    let getframe_name = format!("{}GetFrame", struct_name);
    let free_name = format!("{}Free", struct_name);

    let create_ident = syn::Ident::new(&create_name, struct_name.span());
    let getframe_ident = syn::Ident::new(&getframe_name, struct_name.span());
    let free_ident = syn::Ident::new(&free_name, struct_name.span());

    let create = match arg.to_string().as_str() {
        "video" => {quote! {
            // Filter creation function - the real workhorse
        #[no_mangle]
        pub unsafe extern "C" fn #create_ident(
            in_: *const rustsynth::ffi::VSMap,
            out: *mut rustsynth::ffi::VSMap,
            user_data: *mut std::os::raw::c_void,
            core: *mut rustsynth::ffi::VSCore,
            vsapi: *const rustsynth::ffi::VSAPI,
        ) {
            rustsynth::init_api(vsapi);
            let api = &*vsapi;
                let core_ref = rustsynth::core::CoreRef::from_ptr(core);
                let in_map = rustsynth::map::Map::from_ptr(in_);
                // Create filter instance from arguments
                match #struct_name::from_args(&in_map, &core_ref) {
                    Ok(filter_data) => {
                        let deps = filter_data.get_dependencies();
                        let deps_ffi: Vec<rustsynth::ffi::VSFilterDependency> = deps.iter()
                            .map(|d| d.as_ffi())
                            .collect();

                        // Get filter mode from const
                        let filter_mode = #struct_name::MODE;
                        let media_info = match filter_data.get_video_info() {
                                    Ok(ai) => ai,
                                    Err(error_msg) => {
                                        let error_cstr = std::ffi::CString::new(error_msg).unwrap_or_else(|_| {
                                            std::ffi::CString::new("Failed to video info").unwrap()
                                        });
                                        api.mapSetError.unwrap()(out, error_cstr.as_ptr());
                                        return;
                                    }
                                };

                        // Allocate filter data on heap
                        let data_ptr = Box::into_raw(Box::new(filter_data)) as *mut std::os::raw::c_void;

                                let filter_name = std::ffi::CString::new(#struct_name::NAME).unwrap();
                                api.createVideoFilter.unwrap()(
                                    out,
                                    filter_name.as_ptr(),
                                    &media_info.as_ptr() as *const rustsynth::ffi::VSVideoInfo,
                                    Some(#getframe_ident),
                                    Some(#free_ident),
                                    filter_mode.as_ptr() as i32,
                                    deps_ffi.as_ptr(),
                                    deps_ffi.len() as i32,
                                    data_ptr,
                                    core,
                                );
                        
                    },
                    Err(error_msg) => {
                       eprintln!("{}", error_msg);
                    }
                }
        }
        }},
        "audio" => {quote! {
            // Filter creation function - the real workhorse
        #[no_mangle]
        pub unsafe extern "C" fn #create_ident(
            in_: *const rustsynth::ffi::VSMap,
            out: *mut rustsynth::ffi::VSMap,
            user_data: *mut std::os::raw::c_void,
            core: *mut rustsynth::ffi::VSCore,
            vsapi: *const rustsynth::ffi::VSAPI,
        ) {
            let api = &*vsapi;
            rustsynth::init_api(vsapi);
            std::panic::catch_unwind(|| {
                // Create rustsynth wrapper objects
                let core_ref = rustsynth::core::CoreRef::from_ptr(core);
                let in_map = rustsynth::map::Map::from_ptr(in_);
                
                // Create filter instance from arguments
                match #struct_name::from_args(&in_map, &core_ref) {
                    Ok(filter_data) => {
                        let deps = filter_data.get_dependencies();
                        let deps_ffi: Vec<rustsynth::ffi::VSFilterDependency> = deps.iter()
                            .map(|d| d.as_ffi())
                            .collect();

                        // Get filter mode from const
                        let filter_mode = #struct_name::MODE;
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

                    
                        let filter_name = std::ffi::CString::new(#struct_name::NAME).unwrap();
                        api.createAudioFilter.unwrap()(
                                    out,
                                    filter_name.as_ptr(),
                                    &media_info,
                                    Some(#getframe_ident),
                                    Some(#free_ident),
                                    *filter_mode.as_ptr(),
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
        }}
        _ => return Err(syn::Error::new_spanned(&arg.to_string(), "Unsupported filter type. Use 'video' or 'audio'"))
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
                let filter = &mut *(instance_data as *mut #struct_name);
                let core_ref = rustsynth::core::CoreRef::from_ptr(core);
                let frame_ctx_wrapper = rustsynth::frame::FrameContext::from_ptr(frame_ctx);
                let activation = rustsynth::filter::ActivationReason::from_ffi(activation_reason);

                match activation {
                    rustsynth::filter::ActivationReason::Initial => {
                        // Request the frames we need
                        filter.request_input_frames(n, frame_ctx_wrapper);
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

                        match filter.process_frame(n, frame_data_array, frame_ctx_wrapper, core_ref) {
                            Ok(output_frame) => {
                                // Convert to FrameRef and transfer ownership properly
                                output_frame.into_frame_ref().into_ptr()
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
                    let filter = Box::from_raw(instance_data as *mut #struct_name);
                    filter.cleanup();
                    // Box drop handles memory cleanup
                });
            }
        }

        // Register this filter in the plugin
        impl #struct_name {
            fn register_filter(
                plugin: *mut rustsynth::ffi::VSPlugin,
                vspapi: *const rustsynth::ffi::VSPLUGINAPI
            ) {
                unsafe {
                    let api = &*vspapi;
                    let filter_name = std::ffi::CString::new(#struct_name::NAME).unwrap();
                    let args_spec = std::ffi::CString::new(#struct_name::ARGS).unwrap();
                    let return_spec = std::ffi::CString::new(#struct_name::RETURNTYPE).unwrap();

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
                            eprintln!("Failed to register filter '{}'", #struct_name::NAME);
                        }
                    } else {
                        eprintln!("registerFunction API is NULL - cannot register filter '{}'", #struct_name::NAME);
                    }
                }
            }
        }
    };

    Ok(expanded)
}
