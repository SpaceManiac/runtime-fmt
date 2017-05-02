//! A custom-derive implementation for the `FormatArgs` trait.
extern crate proc_macro;
extern crate syn;
#[macro_use] extern crate quote;

use proc_macro::TokenStream;

/// Derive a `FormatArgs` implementation for the provided input struct.
#[proc_macro_derive(FormatArgs)]
pub fn derive_format_args(input: TokenStream) -> TokenStream {
    let string = input.to_string();
    let ast = syn::parse_derive_input(&string).unwrap();
    implement(&ast).parse().unwrap()
}

fn implement(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let variant = match ast.body {
        syn::Body::Struct(ref variant) => variant,
        _ => panic!("#[derive(FormatArgs)] is not implemented for enums")
    };
    unimplemented!()
}
