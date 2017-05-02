//! A custom-derive implementation for the `FormatArgs` trait.
#![recursion_limit="128"]

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
    // The rough structure of this (dummy_ident, extern crate/use) is based on
    // how serde_derive does it.

    let ident = &ast.ident;
    let variant = match ast.body {
        syn::Body::Struct(ref variant) => variant,
        _ => panic!("#[derive(FormatArgs)] is not implemented for enums")
    };

    let dummy_ident = syn::Ident::new(format!("_IMPL_FORMAT_ARGS_FOR_{}", ident));

    let (validate_name, validate_index, get_child, as_usize);
    match *variant {
        syn::VariantData::Struct(ref fields) => {
            get_child = build_fields(fields);
            as_usize = build_usize(ident, fields);
            validate_index = quote! { false };

            let index = 0..fields.len();
            let ident: Vec<_> = fields.iter()
                .map(|field| field.ident.as_ref().unwrap())
                .map(ToString::to_string)
                .collect();
            validate_name = quote! {
                match name {
                    #(#ident => _Option::Some(#index),)*
                    _ => _Option::None,
                }
            };
        }
        syn::VariantData::Tuple(ref fields) => {
            get_child = build_fields(fields);
            as_usize = build_usize(ident, fields);
            validate_name = quote! { _Option::None };

            let len = fields.len();
            validate_index = quote! { index < #len };
        }
        syn::VariantData::Unit => {
            validate_name = quote! { _Option::None };
            validate_index = quote! { false };
            get_child = quote! { panic!("bad index {}", index) };
            as_usize = get_child.clone();
        }
    };

    // TODO: impl_generics, ty_generics, where_clause
    quote! {
        #[allow(non_upper_case_globals, unused_attributes)]
        #[allow(unused_variables, unused_qualifications)]
        const #dummy_ident: () = {
            extern crate runtime_fmt as _runtime_fmt;
            use std::fmt::{Formatter as _Formatter, Result as _Result};
            use std::option::Option as _Option;
            #[automatically_derived]
            impl _runtime_fmt::FormatArgs for #ident {
                fn validate_name(name: &str) -> _Option<usize> {
                    #validate_name
                }
                fn validate_index(index: usize) -> bool {
                    #validate_index
                }
                fn get_child<__F>(index: usize) -> _Option<fn(&Self, &mut _Formatter) -> _Result>
                    where __F: _runtime_fmt::codegen::FormatTrait + ?Sized
                {
                    #get_child
                }
                fn as_usize(index: usize) -> Option<fn(&Self) -> &usize> {
                    #as_usize
                }
            }
        };
    }
}

fn build_fields(fields: &[syn::Field]) -> quote::Tokens {
    let index = 0..fields.len();
    let ty: Vec<_> = fields.iter().map(|field| &field.ty).collect();
    let ident: Vec<_> = fields.iter().enumerate().map(|(idx, field)| match field.ident {
        Some(ref ident) => ident.clone(),
        None => syn::Ident::from(idx),
    }).collect();
    quote! {
        match index {
            #(
                #index => _runtime_fmt::codegen::combine::<__F, Self, #ty, _>(
                    |this| &this.#ident
                ),
            )*
            _ => panic!("bad index {}", index)
        }
    }
}

fn build_usize(for_: &syn::Ident, fields: &[syn::Field]) -> quote::Tokens {
    let ty_usize = syn::Ty::Path(None, syn::Path {
        global: false,
        segments: vec![syn::PathSegment {
            ident: syn::Ident::from("usize"),
            parameters: syn::PathParameters::AngleBracketed(syn::AngleBracketedParameterData {
                lifetimes: vec![],
                types: vec![],
                bindings: vec![],
            }),
        }]
    });

    let mut result = quote::Tokens::new();
    for (idx, field) in fields.iter().enumerate() {
        // if type is literally `usize`
        if field.ty == ty_usize {
            let ident = &field.ident;
            result.append(quote! {
                #idx => {
                    fn inner(this: &#for_) -> &usize { &this.#ident }
                    Some(inner)
                },
            });
        }
    }

    quote! {
        match index {
            #result
            _ => None
        }
    }
}
