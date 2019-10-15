//! A custom-derive implementation for the `FormatArgs` trait.
#![recursion_limit="128"]
#![feature(option_entry)]

extern crate proc_macro;
extern crate syn;
#[macro_use] extern crate quote;

use proc_macro::TokenStream;

use ast::Container;

mod ast;
mod context;

/// Derive a `FormatArgs` implementation for the provided input struct.
#[proc_macro_derive(FormatArgs, attributes(format_args))]
pub fn derive_format_args(input: TokenStream) -> TokenStream {
    let string = input.to_string();
    let ast = syn::parse_derive_input(&string).unwrap();
    match implement_format_trait(&ast) {
        Ok(tokens) => tokens.parse().unwrap(),
        Err(error) => panic!(error)
    }
}

fn implement_format_trait(ast: &syn::DeriveInput) -> Result<quote::Tokens, String> {
    let container = Container::from_ast(ast)?;

    let validate_name = build_validate_name(&container);
    let validate_index = {
        let max_index = container.fields().len();
        quote! { index < #max_index }
    };
    let get_child = build_get_child(&container);
    let as_usize = build_as_usize(&container);

    let ident = container.ident();
    let dummy_ident = syn::Ident::new(format!("_IMPL_FORMAT_ARGS_FOR_{}", ident));
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    Ok(quote! {
        #[allow(non_upper_case_globals, unused_attributes)]
        #[allow(unused_variables, unused_qualifications)]
        const #dummy_ident: () = {
            extern crate runtime_fmt as _runtime_fmt;
            use std::fmt::{Formatter as _Formatter, Result as _Result};
            use std::option::Option as _Option;
            #[automatically_derived]
            impl #impl_generics _runtime_fmt::FormatArgs for #ident #ty_generics #where_clause {
                fn validate_name(name: &str) -> _Option<usize> {
                    #validate_name
                }
                #[allow(unused_comparisons)]
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
    })
}

fn build_validate_name<'a>(container: &Container<'a>) -> quote::Tokens {
    let mut matches = quote::Tokens::new();
    for field in container.fields() {
        let aliases = field.aliases();
        if !aliases.is_empty() {
            let index = field.index();
            matches.append(quote! { #(#aliases)|* => _Option::Some(#index), });
        }
    }
    quote! {
        match name {
            #matches
            _ => _Option::None
        }
    }
}

fn build_get_child<'a>(container: &Container<'a>) -> quote::Tokens {
    let mut matches = quote::Tokens::new();
    for field in container.fields() {
        let index = field.index();
        let ty = field.ty();
        let ident = field.ident();

        matches.append(quote! {
            #index => _runtime_fmt::codegen::combine::<__F, Self, #ty, _>(
                |this| &this.#ident
            ),
        });
    }
    quote! {
        match index {
            #matches
            _ => panic!("Bad index: {}", index)
        }
    }
}

fn build_as_usize<'a>(container: &'a Container) -> quote::Tokens {
    let self_ = container.ident();
    let (_, ty_generics, where_clause) = container.generics().split_for_impl();

    // To avoid causing trouble with lifetime elision rules, an explicit
    // lifetime for the input and output is used.
    let lifetime = syn::Ident::new("'__as_usize_inner");
    let mut generics2 = container.generics().clone();
    generics2.lifetimes.insert(0, syn::LifetimeDef {
        attrs: vec![],
        lifetime: syn::Lifetime { ident: lifetime.clone() },
        bounds: vec![],
    });
    let (impl_generics, _, _) = generics2.split_for_impl();

    let mut matches = quote::Tokens::new();
    for field in container.fields() {
        let index = field.index();
        let ty = field.ty();
        let ident = field.ident();

        matches.append(quote! {
            #index => {
                fn inner #impl_generics (this: &#lifetime #self_ #ty_generics)
                    -> &#lifetime #ty
                    #where_clause { &this.#ident }
                _runtime_fmt::codegen::as_usize(inner)
            },
        });
    }

    quote! {
        match index {
            #matches
            _ => panic!("bad index {}", index)
        }
    }
}