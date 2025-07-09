extern crate proc_macro2;
extern crate quote;
extern crate syn;

mod attributes;
mod enum_with_primitive;
mod enums;
mod field_attrs;
mod shared;
mod simple_enum;
mod structs;

use enum_with_primitive::enum_with_primitive_serializer;
use enums::enum_serializer;
use proc_macro2::TokenStream;
use quote::quote;
use simple_enum::simple_enum_serializer;
use structs::struct_serializer;
use syn::{Data, DeriveInput, Fields, Ident, parse_macro_input};

/// available attributes:
/// - `#[primitive(PrimitiveName)]` - set primitive enum for complex enum
/// - `#[idx = 1]`
/// - `#[rapira(static_size = expr)]`
/// - `#[rapira(min_size = expr)]`
/// - `#[rapira(with = path)]`
/// - `#[rapira(skip)]`
/// - `#[rapira(debug)]`
#[proc_macro_derive(Rapira, attributes(rapira, idx, primitive))]
pub fn serializer_trait(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(stream as DeriveInput);
    let name = &ast.ident;
    let data = &ast.data;
    let is_debug = attributes::debug_attr(&ast.attrs);

    match data {
        Data::Struct(data_struct) => struct_serializer(data_struct, name, ast.generics, is_debug),
        Data::Enum(data_enum) => {
            let is_simple_enum = data_enum.variants.iter().all(|item| item.fields.is_empty());
            if is_simple_enum {
                simple_enum_serializer(name)
            } else {
                let primitive_name = attributes::get_primitive_name(&ast.attrs);

                match primitive_name {
                    Some(primitive_name) => {
                        enum_with_primitive_serializer(data_enum, name, primitive_name)
                    }
                    None => {
                        let enum_static_size = attributes::enum_static_size(&ast.attrs);
                        let min_size = attributes::min_size(&ast.attrs);
                        enum_serializer(
                            data_enum,
                            name,
                            enum_static_size,
                            min_size,
                            ast.generics,
                            is_debug,
                        )
                    }
                }
            }
        }
        Data::Union(_) => {
            panic!(
                "unions not supported, but Rust enums is implemented Rapira trait (use Enums instead)"
            );
        }
    }
}

/// #[primitive(PrimitiveName)]
#[proc_macro_derive(PrimitiveFromEnum, attributes(primitive))]
pub fn derive_primitive_from_enum(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(stream as DeriveInput);

    let name = &ast.ident;
    let data = &ast.data;

    match data {
        Data::Enum(data_enum) => {
            let is_simple_enum = data_enum.variants.iter().all(|item| item.fields.is_empty());

            if is_simple_enum {
                panic!("PrimitiveFromEnum only for non simple enum allow");
            } else {
                let primitive_name = ast
                    .attrs
                    .iter()
                    .find_map(|attr| {
                        if !attr.path().is_ident("primitive") {
                            return None;
                        }

                        let ident: Ident = attr.parse_args().unwrap();

                        Some(ident)
                    })
                    .expect("complex enums must include primitive type name");

                let len = data_enum.variants.len();

                let mut get_primitive_enum: Vec<TokenStream> = Vec::with_capacity(len);

                for variant in &data_enum.variants {
                    let variant_name = &variant.ident;

                    match &variant.fields {
                        Fields::Unit => {
                            get_primitive_enum.push(quote! {
                                #name::#variant_name => #primitive_name::#variant_name,
                            });
                        }
                        Fields::Unnamed(fields) => {
                            let len = fields.unnamed.len();
                            if len == 1 {
                                get_primitive_enum.push(quote! {
                                    #name::#variant_name(_) => #primitive_name::#variant_name,
                                });
                            } else {
                                let underscores = vec![quote! { ,_ }; len - 1];
                                get_primitive_enum.push(quote! {
                                    #name::#variant_name(_ #(#underscores)*) => #primitive_name::#variant_name,
                                });
                            }
                        }
                        Fields::Named(fields) => {
                            let fields = &fields
                                .named
                                .iter()
                                .map(|f| {
                                    let ident = f.ident.as_ref().unwrap();
                                    quote! { #ident: _, }
                                })
                                .collect::<Vec<_>>();
                            get_primitive_enum.push(quote! {
                                #name::#variant_name{ #(#fields)* } => #primitive_name::#variant_name,
                            });
                        }
                    };
                }

                let res = quote! {
                    impl From<&#name> for #primitive_name {
                        #[inline]
                        fn from(value: &#name) -> Self {
                            match value {
                                #(#get_primitive_enum)*
                            }
                        }
                    }
                };

                proc_macro::TokenStream::from(res)
            }
        }
        _ => {
            panic!("PrimitiveFromEnum only for enum allow");
        }
    }
}

#[proc_macro_derive(FromU8, attributes(primitive))]
pub fn derive_from_u8(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(stream as DeriveInput);

    let name = &ast.ident;
    let data = &ast.data;

    match data {
        Data::Enum(data_enum) => {
            let is_simple_enum = data_enum.variants.iter().all(|item| item.fields.is_empty());
            if is_simple_enum {
                let mut variants: Vec<TokenStream> = Vec::with_capacity(data_enum.variants.len());
                let mut try_variants: Vec<TokenStream> =
                    Vec::with_capacity(data_enum.variants.len());

                for variant in &data_enum.variants {
                    let ident = &variant.ident;
                    let var = quote! {
                        u if #name::#ident == u => #name::#ident,
                    };
                    variants.push(var);
                    try_variants.push(quote! {
                        u if #name::#ident == u => Ok(#name::#ident),
                    });
                }

                let r#gen = quote! {
                    impl PartialEq<u8> for #name {
                        fn eq(&self, other: &u8) -> bool {
                            *self as u8 == *other
                        }
                    }
                    impl From<#name> for u8 {
                        fn from(e: #name) -> u8 {
                            e as u8
                        }
                    }
                    impl rapira::FromU8 for #name {
                        /// # Panics
                        ///
                        /// Panics if `u` is not equal to any variant
                        #[inline]
                        fn from_u8(u: u8) -> Self {
                            match u {
                                #(#variants)*
                                _ => panic!("FromU8 undefined value: {}", u),
                            }
                        }
                    }
                    impl core::convert::TryFrom<u8> for #name {
                        type Error = rapira::EnumFromU8Error;
                        fn try_from(value: u8) -> Result<Self, Self::Error> {
                            match value {
                                #(#try_variants)*
                                _ => Err(rapira::EnumFromU8Error),
                            }
                        }
                    }
                };
                proc_macro::TokenStream::from(r#gen)
            } else {
                panic!("FromU8 only for simple enum allow (without nested data)");
            }
        }
        _ => {
            panic!("FromU8 only for enum allow");
        }
    }
}
