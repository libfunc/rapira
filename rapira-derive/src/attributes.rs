extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Attribute, Expr, Ident, Lit, Meta, MetaNameValue, NestedMeta};

pub fn get_primitive_name(attrs: &[Attribute]) -> Option<TokenStream> {
    attrs.iter().find_map(|attr| {
        attr.path.segments.first().and_then(|segment| {
            if segment.ident != "primitive" {
                return None;
            }
            match attr.parse_meta() {
                Ok(Meta::NameValue(name_value)) => {
                    if let Lit::Str(litstr) = name_value.lit {
                        let s = litstr.parse::<Ident>().unwrap();
                        let value = s.to_token_stream();
                        Some(value)
                    } else {
                        None
                    }
                }
                Ok(_) => None,
                Err(_) => None,
            }
        })
    })
}

pub fn enum_static_size(attrs: &[Attribute]) -> Option<Expr> {
    attrs.iter().find_map(|attr| {
        attr.path.segments.first().and_then(|segment| {
            if segment.ident != "rapira" {
                return None;
            }
            match attr.parse_meta() {
                Ok(Meta::List(list)) => list.nested.iter().find_map(|nested| match nested {
                    NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                        path,
                        lit: Lit::Str(lit_str),
                        ..
                    })) => path.segments.first().and_then(|segment| {
                        if segment.ident != "static_size" {
                            return None;
                        }
                        match lit_str.parse::<Expr>() {
                            Ok(expr) => Some(expr),
                            Err(_) => {
                                panic!("invalid path");
                            }
                        }
                    }),
                    _ => None,
                }),
                Ok(_) => None,
                Err(_) => None,
            }
        })
    })
}
