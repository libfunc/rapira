extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Attribute, Ident, Lit, Meta};

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
