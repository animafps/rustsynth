use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Ident};


/// A consuming procedual macro to provide a method to turn a struct to a OwnedMap
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
        syn::Data::Struct(ds) => {
            match &ds.fields {
                syn::Fields::Named(named) => {
                    named.named.iter().map(|x| x.ident.clone().unwrap()).collect()
                },
                _ => unreachable!()
            }
        },
        _ => unreachable!(),
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