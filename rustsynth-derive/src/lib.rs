use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use rustsynth::core::{CoreCreationFlags, CoreRef};
use syn::{self, Ident};

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