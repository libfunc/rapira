extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, Data, DeriveInput, Field, Fields, Ident, Lit, LitInt, Meta, MetaNameValue,
    NestedMeta,
};

fn get_primitive_name(ast: &DeriveInput) -> Option<TokenStream> {
    ast.attrs.iter().find_map(|attr| {
        attr.path.segments.first().and_then(|segment| {
            if segment.ident != "primitive" {
                return None;
            }
            match attr.parse_args::<MetaNameValue>() {
                Ok(name_value) => {
                    if name_value.path.to_token_stream().to_string() != "name" {
                        return None;
                    }
                    if let Lit::Str(litstr) = name_value.lit {
                        let s = litstr.parse::<Ident>().unwrap();
                        let value = s.to_token_stream();
                        Some(value)
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        })
    })
}

#[proc_macro_derive(Rapira, attributes(idx, coming))]
pub fn serializer_trait(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(stream as DeriveInput);
    let name = &ast.ident;
    let data = &ast.data;
    match data {
        Data::Struct(data_struct) => {
            let fields = &data_struct.fields;
            match fields {
                Fields::Named(fields) => {
                    let named = &fields.named;
                    let named_len = named.len();

                    let mut fields_insert: Vec<(Field, u32)> = Vec::with_capacity(named_len);
                    let mut seq = 0u32;

                    for field in named.iter() {
                        let field_idx = field
                            .attrs
                            .iter()
                            .find_map(|a| {
                                a.path.segments.first().and_then(|segment| {
                                    if segment.ident != "idx" {
                                        return None;
                                    }
                                    match a.parse_args::<Meta>() {
                                        Ok(Meta::List(list)) => {
                                            let a = list.nested.first().unwrap();
                                            let int: u32 = match a {
                                                NestedMeta::Lit(Lit::Int(i)) => {
                                                    i.base10_parse::<u32>().unwrap()
                                                }
                                                _ => {
                                                    panic!("error meta type")
                                                }
                                            };
                                            Some(int)
                                        }
                                        Ok(_) => None,
                                        Err(_) => None,
                                    }
                                })
                            })
                            .unwrap_or_else(|| {
                                let current_seq = seq;
                                seq += 1;
                                current_seq
                            });

                        fields_insert.push((field.clone(), field_idx));
                    }

                    fields_insert.sort_by(|(_, idx_a), (_, idx_b)| idx_a.cmp(idx_b));

                    let try_convert_to_bytes: Vec<TokenStream> = fields_insert
                        .iter()
                        .map(|(field, _)| {
                            let ident = field.ident.as_ref().unwrap();

                            let gen = quote! {
                               self.#ident.try_convert_to_bytes(slice, cursor)?;
                            };

                            gen
                        })
                        .collect();

                    let convert_to_bytes: Vec<TokenStream> = fields_insert
                        .iter()
                        .map(|(field, _)| {
                            let ident = field.ident.as_ref().unwrap();

                            let gen = quote! {
                               self.#ident.convert_to_bytes(slice, cursor);
                            };

                            gen
                        })
                        .collect();

                    let from_slice: Vec<TokenStream> = fields_insert
                        .iter()
                        .map(|(field, _)| {
                            let ident = field.ident.as_ref().unwrap();
                            let typ = &field.ty;

                            let gen = quote! {
                                let #ident = <#typ as rapira::Rapira>::from_slice(slice)?;
                            };

                            gen
                        })
                        .collect();

                    let check_bytes: Vec<TokenStream> = fields_insert
                        .iter()
                        .map(|(field, _)| {
                            let typ = &field.ty;
                            quote! {
                                <#typ>::check_bytes(slice)?;
                            }
                        })
                        .collect();

                    let from_slice_unchecked: Vec<TokenStream> = fields_insert
                        .iter()
                        .map(|(field, _)| {
                            let ident = field.ident.as_ref().unwrap();
                            let typ = &field.ty;

                            let gen = quote! {
                                let #ident = <#typ>::from_slice_unchecked(slice)?;
                            };

                            gen
                        })
                        .collect();

                    let from_slice_unsafe: Vec<TokenStream> = fields_insert
                        .iter()
                        .map(|(field, _)| {
                            let ident = field.ident.as_ref().unwrap();
                            let typ = &field.ty;

                            let gen = quote! {
                                let #ident = <#typ>::from_slice_unsafe(slice)?;
                            };

                            gen
                        })
                        .collect();

                    let field_names: Vec<TokenStream> = fields_insert
                        .iter()
                        .map(|(field, _)| {
                            let ident = field.ident.as_ref().unwrap();
                            quote! { #ident, }
                        })
                        .collect();

                    let size: Vec<TokenStream> = fields_insert
                        .iter()
                        .map(|(field, _)| {
                            let ident = field.ident.as_ref().unwrap();
                            let typ = &field.ty;

                            quote! { + (match <#typ>::STATIC_SIZE {
                                Some(s) => s,
                                None => self.#ident.size()
                            }) }
                        })
                        .collect();

                    let static_sizes: Vec<TokenStream> = fields_insert
                        .iter()
                        .map(|(field, _)| {
                            let typ = &field.ty;
                            quote! { <#typ>::STATIC_SIZE, }
                        })
                        .collect();

                    let gen = quote! {
                        impl rapira::Rapira for #name {
                            const STATIC_SIZE: Option<usize> = rapira::static_size([#(#static_sizes)*]);

                            #[inline]
                            fn from_slice(slice: &mut &[u8]) -> Result<Self, rapira::RapiraError>
                            where
                                Self: Sized,
                            {
                                #(#from_slice)*
                                Ok(#name {
                                    #(#field_names)*
                                })
                            }

                            #[inline]
                            fn check_bytes(slice: &mut &[u8]) -> Result<(), rapira::RapiraError>
                            where
                                Self: Sized,
                            {
                                #(#check_bytes)*
                                Ok(())
                            }

                            #[inline]
                            fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self, rapira::RapiraError>
                            where
                                Self: Sized,
                            {
                                #(#from_slice_unchecked)*
                                Ok(#name {
                                    #(#field_names)*
                                })
                            }

                            #[inline]
                            unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self, rapira::RapiraError>
                            where
                                Self: Sized,
                            {
                                #(#from_slice_unsafe)*
                                Ok(#name {
                                    #(#field_names)*
                                })
                            }

                            #[inline]
                            fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<(), rapira::RapiraError> {
                                #(#try_convert_to_bytes)*
                                Ok(())
                            }

                            #[inline]
                            fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
                                #(#convert_to_bytes)*
                            }

                            #[inline]
                            fn size(&self) -> usize {
                                0 #(#size)*
                            }
                        }
                    };
                    proc_macro::TokenStream::from(gen)
                }
                Fields::Unnamed(fields) => {
                    let unnamed = &fields.unnamed;
                    let unnamed_len = unnamed.len();
                    let mut field_names: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
                    let mut from_slice: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
                    let mut check_bytes: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
                    let mut from_slice_unchecked: Vec<TokenStream> =
                        Vec::with_capacity(unnamed_len);
                    let mut from_slice_unsafe: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
                    let mut try_convert_to_bytes: Vec<TokenStream> =
                        Vec::with_capacity(unnamed_len);
                    let mut convert_to_bytes: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
                    let mut size: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
                    let mut static_sizes: Vec<TokenStream> = Vec::with_capacity(unnamed_len);

                    for (idx, field) in unnamed.iter().enumerate() {
                        let id = syn::Lit::Int(LitInt::new(&idx.to_string(), Span::call_site()));
                        let typ = &field.ty;
                        let field_name = syn::Ident::new(&format!("arg{}", idx), Span::call_site());
                        let field_name_into = quote! { #field_name, };

                        from_slice.push(quote! {
                            let #field_name = <#typ>::from_slice(slice)?;
                        });
                        check_bytes.push(quote! {
                            <#typ>::check_bytes(slice)?;
                        });
                        from_slice_unchecked.push(quote! {
                            let #field_name = <#typ>::from_slice_unchecked(slice)?;
                        });
                        from_slice_unsafe.push(quote! {
                            let #field_name = <#typ>::from_slice_unsafe(slice)?;
                        });
                        try_convert_to_bytes.push(quote! {
                            self.#id.try_convert_to_bytes(slice, cursor)?;
                        });
                        convert_to_bytes.push(quote! {
                            self.#id.convert_to_bytes(slice, cursor);
                        });
                        size.push(quote! { + (match <#typ>::STATIC_SIZE {
                            Some(s) => s,
                            None => self.#id.size()
                        }) });
                        static_sizes.push(quote! {
                            <#typ>::STATIC_SIZE,
                        });
                        field_names.push(field_name_into);
                    }

                    let gen = quote! {
                        impl rapira::Rapira for #name {
                            const STATIC_SIZE: Option<usize> = rapira::static_size([#(#static_sizes)*]);

                            #[inline]
                            fn from_slice(slice: &mut &[u8]) -> Result<Self, rapira::RapiraError>
                            where
                                Self: Sized,
                            {
                                #(#from_slice)*
                                Ok(#name(#(#field_names)*))
                            }

                            #[inline]
                            fn check_bytes(slice: &mut &[u8]) -> Result<(), rapira::RapiraError>
                            where
                                Self: Sized,
                            {
                                #(#check_bytes)*
                                Ok(())
                            }

                            #[inline]
                            fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self, rapira::RapiraError>
                            where
                                Self: Sized,
                            {
                                #(#from_slice_unchecked)*
                                Ok(#name(#(#field_names)*))
                            }

                            #[inline]
                            unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self, rapira::RapiraError>
                            where
                                Self: Sized,
                            {
                                #(#from_slice_unsafe)*
                                Ok(#name(#(#field_names)*))
                            }

                            #[inline]
                            fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<(), rapira::RapiraError> {
                                #(#try_convert_to_bytes)*
                                Ok(())
                            }

                            #[inline]
                            fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
                                #(#convert_to_bytes)*
                            }

                            #[inline]
                            fn size(&self) -> usize { 0 #(#size)* }
                        }
                    };

                    proc_macro::TokenStream::from(gen)
                }
                Fields::Unit => {
                    let gen = quote! {
                        impl rapira::Rapira for #name {
                            const STATIC_SIZE: Option<usize> = Some(0);

                            #[inline]
                            fn from_slice(slice: &mut &[u8]) -> Result<Self, rapira::RapiraError>
                            where
                                Self: Sized,
                            {
                                Ok(#name)
                            }

                            #[inline]
                            fn check_bytes(slice: &mut &[u8]) -> Result<(), rapira::RapiraError>
                            where
                                Self: Sized,
                            {
                                Ok(())
                            }

                            #[inline]
                            fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self, rapira::RapiraError>
                            where
                                Self: Sized,
                            {
                                Ok(#name)
                            }

                            #[inline]
                            unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self, rapira::RapiraError>
                            where
                                Self: Sized,
                            {
                                Ok(#name)
                            }

                            #[inline]
                            fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<(), rapira::RapiraError> {
                                Ok(())
                            }

                            #[inline]
                            fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {}

                            #[inline]
                            fn size(&self) -> usize { 0 }
                        }
                    };
                    proc_macro::TokenStream::from(gen)
                }
            }
        }
        Data::Enum(data_enum) => {
            let is_simple_enum = data_enum.variants.iter().all(|item| item.fields.is_empty());
            let variants_len = data_enum.variants.len();

            if is_simple_enum {
                let gen = quote! {
                    impl rapira::Rapira for #name {
                        const STATIC_SIZE: Option<usize> = Some(1);

                        #[inline]
                        fn from_slice(slice: &mut &[u8]) -> Result<Self, rapira::RapiraError>
                        where
                            Self: Sized,
                        {
                            let val: u8 = u8::from_slice(slice)?;
                            <Self as core::convert::TryFrom<u8>>::try_from(val).map_err(|_| rapira::RapiraError::EnumVariantError)
                        }

                        #[inline]
                        fn check_bytes(slice: &mut &[u8]) -> Result<(), rapira::RapiraError>
                        where
                            Self: Sized,
                        {
                            let val: u8 = u8::from_slice(slice)?;
                            <Self as core::convert::TryFrom<u8>>::try_from(val).map_err(|_| rapira::RapiraError::EnumVariantError)?;
                            Ok(())
                        }

                        #[inline]
                        fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self, rapira::RapiraError>
                        where
                            Self: Sized,
                        {
                            let val: u8 = u8::from_slice(slice)?;
                            <Self as core::convert::TryFrom<u8>>::try_from(val).map_err(|_| rapira::RapiraError::EnumVariantError)
                        }

                        #[inline]
                        unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self, rapira::RapiraError>
                        where
                            Self: Sized,
                        {
                            let val: u8 = u8::from_slice_unsafe(slice)?;
                            <Self as core::convert::TryFrom<u8>>::try_from(val).map_err(|_| rapira::RapiraError::EnumVariantError)
                        }

                        #[inline]
                        fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<(), rapira::RapiraError> {
                            rapira::push(slice, cursor, *self as u8);
                            Ok(())
                        }

                        #[inline]
                        fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
                            rapira::push(slice, cursor, *self as u8);
                        }
                        #[inline]
                        fn size(&self) -> usize { 1 }
                    }
                };

                proc_macro::TokenStream::from(gen)
            } else {
                let primitive_name = get_primitive_name(&ast);

                match primitive_name {
                    Some(primitive_name) => {
                        let mut from_slice: Vec<TokenStream> = Vec::with_capacity(variants_len);
                        let mut check_bytes: Vec<TokenStream> = Vec::with_capacity(variants_len);
                        let mut from_slice_unchecked: Vec<TokenStream> =
                            Vec::with_capacity(variants_len);
                        let mut from_slice_unsafe: Vec<TokenStream> =
                            Vec::with_capacity(variants_len);
                        let mut try_convert_to_bytes: Vec<TokenStream> =
                            Vec::with_capacity(variants_len);
                        let mut convert_to_bytes: Vec<TokenStream> =
                            Vec::with_capacity(variants_len);
                        let mut size: Vec<TokenStream> = Vec::with_capacity(variants_len);
                        let mut enum_sizes: Vec<TokenStream> = Vec::with_capacity(variants_len);

                        for variant in &data_enum.variants {
                            let variant_name = &variant.ident;
                            match &variant.fields {
                                Fields::Unit => {
                                    from_slice.push(quote! {
                                        #primitive_name::#variant_name => {
                                            Ok(#name::#variant_name)
                                        }
                                    });

                                    check_bytes.push(quote! {
                                        #primitive_name::#variant_name => {}
                                    });

                                    from_slice_unchecked.push(quote! {
                                        #primitive_name::#variant_name => {
                                            Ok(#name::#variant_name)
                                        }
                                    });

                                    from_slice_unsafe.push(quote! {
                                        #primitive_name::#variant_name => {
                                            Ok(#name::#variant_name)
                                        }
                                    });

                                    try_convert_to_bytes.push(quote! {
                                        #name::#variant_name => {}
                                    });

                                    convert_to_bytes.push(quote! {
                                        #name::#variant_name => {}
                                    });

                                    size.push(quote! {
                                        #name::#variant_name => 0,
                                    });

                                    enum_sizes.push(quote! {
                                        None,
                                    });
                                }
                                Fields::Unnamed(fields) => {
                                    let len = fields.unnamed.len();
                                    if len == 1 {
                                        let field = fields.unnamed.first().unwrap();
                                        let typ = &field.ty;

                                        from_slice.push(quote! {
                                            #primitive_name::#variant_name => {
                                                let v = <#typ>::from_slice(slice)?;
                                                Ok(#name::#variant_name(v))
                                            }
                                        });

                                        check_bytes.push(quote! {
                                            #primitive_name::#variant_name => {
                                                <#typ>::check_bytes(slice)?;
                                            }
                                        });

                                        from_slice_unchecked.push(quote! {
                                            #primitive_name::#variant_name => {
                                                let v = <#typ>::from_slice_unchecked(slice)?;
                                                Ok(#name::#variant_name(v))
                                            }
                                        });

                                        from_slice_unsafe.push(quote! {
                                            #primitive_name::#variant_name => {
                                                let v = <#typ>::from_slice_unsafe(slice)?;
                                                Ok(#name::#variant_name(v))
                                            }
                                        });

                                        try_convert_to_bytes.push(quote! {
                                            #name::#variant_name(v) => {
                                                v.try_convert_to_bytes(slice, cursor)?;
                                            }
                                        });

                                        convert_to_bytes.push(quote! {
                                            #name::#variant_name(v) => {
                                                v.convert_to_bytes(slice, cursor);
                                            }
                                        });

                                        size.push(quote! {
                                            #name::#variant_name(v) => {
                                                match <#typ>::STATIC_SIZE {
                                                    Some(s) => s,
                                                    None => v.size(),
                                                }
                                            },
                                        });

                                        enum_sizes.push(quote! {
                                            <#typ>::STATIC_SIZE,
                                        });
                                    } else {
                                        let unnamed = &fields.unnamed;

                                        let mut field_names: Vec<TokenStream> =
                                            Vec::with_capacity(len);
                                        let mut unnamed_from_slice: Vec<TokenStream> =
                                            Vec::with_capacity(len);
                                        let mut unnamed_check_bytes: Vec<TokenStream> =
                                            Vec::with_capacity(len);
                                        let mut unnamed_from_slice_unchecked: Vec<TokenStream> =
                                            Vec::with_capacity(len);
                                        let mut unnamed_from_slice_unsafe: Vec<TokenStream> =
                                            Vec::with_capacity(len);
                                        let mut unnamed_try_convert_to_bytes: Vec<TokenStream> =
                                            Vec::with_capacity(len);
                                        let mut unnamed_convert_to_bytes: Vec<TokenStream> =
                                            Vec::with_capacity(len);
                                        let mut unnamed_size: Vec<TokenStream> =
                                            Vec::with_capacity(len);
                                        let mut unnamed_static_sizes: Vec<TokenStream> =
                                            Vec::with_capacity(len);

                                        for (idx, field) in unnamed.iter().enumerate() {
                                            let typ = &field.ty;
                                            let field_name = syn::Ident::new(
                                                &format!("arg{}", idx),
                                                Span::call_site(),
                                            );

                                            unnamed_from_slice.push(quote! {
                                                let #field_name = <#typ>::from_slice(slice)?;
                                            });
                                            unnamed_check_bytes.push(quote! {
                                                <#typ>::check_bytes(slice)?;
                                            });
                                            unnamed_from_slice_unchecked.push(quote! {
                                                let #field_name = <#typ>::from_slice_unchecked(slice)?;
                                            });
                                            unnamed_from_slice_unsafe.push(quote! {
                                                let #field_name = <#typ>::from_slice_unsafe(slice)?;
                                            });
                                            unnamed_try_convert_to_bytes.push(quote! {
                                                #field_name.try_convert_to_bytes(slice, cursor)?;
                                            });
                                            unnamed_convert_to_bytes.push(quote! {
                                                #field_name.convert_to_bytes(slice, cursor);
                                            });
                                            unnamed_size.push(
                                                quote! { + (match <#typ>::STATIC_SIZE {
                                                    Some(s) => s,
                                                    None => #field_name.size()
                                                }) },
                                            );
                                            unnamed_static_sizes.push(quote! {
                                                <#typ>::STATIC_SIZE,
                                            });
                                            field_names.push(quote! { #field_name, });
                                        }

                                        from_slice.push(quote! {
                                            #primitive_name::#variant_name => {
                                                #(#unnamed_from_slice)*
                                                Ok(#name::#variant_name(#(#field_names)*))
                                            }
                                        });

                                        check_bytes.push(quote! {
                                            #primitive_name::#variant_name => {
                                                #(#unnamed_check_bytes)*
                                            }
                                        });

                                        from_slice_unchecked.push(quote! {
                                            #primitive_name::#variant_name => {
                                                #(#unnamed_from_slice_unchecked)*
                                                Ok(#name::#variant_name(#(#field_names)*))
                                            }
                                        });

                                        from_slice_unsafe.push(quote! {
                                            #primitive_name::#variant_name => {
                                                #(#unnamed_from_slice_unsafe)*
                                                Ok(#name::#variant_name(#(#field_names)*))
                                            }
                                        });

                                        try_convert_to_bytes.push(quote! {
                                            #name::#variant_name(#(#field_names)*) => {
                                                #(#unnamed_try_convert_to_bytes)*
                                            }
                                        });

                                        convert_to_bytes.push(quote! {
                                            #name::#variant_name(#(#field_names)*) => {
                                                #(#unnamed_convert_to_bytes)*
                                            }
                                        });

                                        size.push(quote! {
                                            #name::#variant_name(#(#field_names)*) => {
                                                0 #(#unnamed_size)*
                                            },
                                        });

                                        enum_sizes.push(quote! {
                                            rapira::static_size([#(#unnamed_static_sizes)*]),
                                        });
                                    }
                                }
                                Fields::Named(fields) => {
                                    let named = &fields.named;
                                    let len = named.len();

                                    let mut fields_insert: Vec<(Field, u32)> =
                                        Vec::with_capacity(len);
                                    let mut seq = 0u32;

                                    for field in named.iter() {
                                        let field_idx = field
                                            .attrs
                                            .iter()
                                            .find_map(|a| {
                                                a.path.segments.first().and_then(|segment| {
                                                    if segment.ident != "idx" {
                                                        return None;
                                                    }
                                                    match a.parse_args::<Meta>() {
                                                        Ok(Meta::List(list)) => {
                                                            let a = list.nested.first().unwrap();
                                                            let int: u32 = match a {
                                                                NestedMeta::Lit(Lit::Int(i)) => {
                                                                    i.base10_parse::<u32>().unwrap()
                                                                }
                                                                _ => {
                                                                    panic!("error meta type")
                                                                }
                                                            };
                                                            Some(int)
                                                        }
                                                        Ok(_) => None,
                                                        Err(_) => None,
                                                    }
                                                })
                                            })
                                            .unwrap_or_else(|| {
                                                let current_seq = seq;
                                                seq += 1;
                                                current_seq
                                            });

                                        fields_insert.push((field.clone(), field_idx));
                                    }

                                    fields_insert
                                        .sort_by(|(_, idx_a), (_, idx_b)| idx_a.cmp(idx_b));

                                    let mut field_names: Vec<TokenStream> = Vec::with_capacity(len);
                                    let mut named_from_slice: Vec<TokenStream> =
                                        Vec::with_capacity(len);
                                    let mut named_check_bytes: Vec<TokenStream> =
                                        Vec::with_capacity(len);
                                    let mut named_from_slice_unchecked: Vec<TokenStream> =
                                        Vec::with_capacity(len);
                                    let mut named_from_slice_unsafe: Vec<TokenStream> =
                                        Vec::with_capacity(len);
                                    let mut named_try_convert_to_bytes: Vec<TokenStream> =
                                        Vec::with_capacity(len);
                                    let mut named_convert_to_bytes: Vec<TokenStream> =
                                        Vec::with_capacity(len);
                                    let mut named_size: Vec<TokenStream> = Vec::with_capacity(len);
                                    let mut named_static_sizes: Vec<TokenStream> =
                                        Vec::with_capacity(len);

                                    for field in fields_insert.iter().map(|(f, _)| f) {
                                        let typ = &field.ty;
                                        let field_name = field.ident.as_ref().unwrap();

                                        named_from_slice.push(quote! {
                                            let #field_name = <#typ>::from_slice(slice)?;
                                        });
                                        named_check_bytes.push(quote! {
                                            <#typ>::check_bytes(slice)?;
                                        });
                                        named_from_slice_unchecked.push(quote! {
                                            let #field_name = <#typ>::from_slice_unchecked(slice)?;
                                        });
                                        named_from_slice_unsafe.push(quote! {
                                            let #field_name = <#typ>::from_slice_unsafe(slice)?;
                                        });
                                        named_try_convert_to_bytes.push(quote! {
                                            #field_name.try_convert_to_bytes(slice, cursor)?;
                                        });
                                        named_convert_to_bytes.push(quote! {
                                            #field_name.convert_to_bytes(slice, cursor);
                                        });
                                        named_size.push(quote! { + (match <#typ>::STATIC_SIZE {
                                            Some(s) => s,
                                            None => #field_name.size()
                                        }) });
                                        named_static_sizes.push(quote! {
                                            <#typ>::STATIC_SIZE,
                                        });
                                        field_names.push(quote! { #field_name, });
                                    }

                                    from_slice.push(quote! {
                                        #primitive_name::#variant_name => {
                                            #(#named_from_slice)*
                                            Ok(#name::#variant_name{#(#field_names)*})
                                        }
                                    });

                                    check_bytes.push(quote! {
                                        #primitive_name::#variant_name => {
                                            #(#named_check_bytes)*
                                        }
                                    });

                                    from_slice_unchecked.push(quote! {
                                        #primitive_name::#variant_name => {
                                            #(#named_from_slice_unchecked)*
                                            Ok(#name::#variant_name{#(#field_names)*})
                                        }
                                    });

                                    from_slice_unsafe.push(quote! {
                                        #primitive_name::#variant_name => {
                                            #(#named_from_slice_unsafe)*
                                            Ok(#name::#variant_name{#(#field_names)*})
                                        }
                                    });

                                    try_convert_to_bytes.push(quote! {
                                        #name::#variant_name{#(#field_names)*} => {
                                            #(#named_try_convert_to_bytes)*
                                        }
                                    });

                                    convert_to_bytes.push(quote! {
                                        #name::#variant_name{#(#field_names)*} => {
                                            #(#named_convert_to_bytes)*
                                        }
                                    });

                                    size.push(quote! {
                                        #name::#variant_name{#(#field_names)*} => {
                                            0 #(#named_size)*
                                        },
                                    });

                                    enum_sizes.push(quote! {
                                        rapira::static_size([#(#named_static_sizes)*]),
                                    });
                                }
                            };
                        }

                        let gen = quote! {
                            impl rapira::Rapira for #name {
                                const STATIC_SIZE: Option<usize> = rapira::enum_size([#(#enum_sizes)*]);

                                #[inline]
                                fn from_slice(slice: &mut &[u8]) -> Result<Self, rapira::RapiraError>
                                where
                                    Self: Sized,
                                {
                                    let val: u8 = u8::from_slice(slice)?;
                                    let t = <#primitive_name as core::convert::TryFrom<u8>>::try_from(val).map_err(|_| rapira::RapiraError::EnumVariantError)?;
                                    match t {
                                        #(#from_slice)*
                                    }
                                }

                                #[inline]
                                fn check_bytes(slice: &mut &[u8]) -> Result<(), rapira::RapiraError>
                                where
                                    Self: Sized,
                                {
                                    let val: u8 = u8::from_slice(slice)?;
                                    let t = <#primitive_name as core::convert::TryFrom<u8>>::try_from(val).map_err(|_| rapira::RapiraError::EnumVariantError)?;
                                    match t {
                                        #(#check_bytes)*
                                    }
                                    Ok(())
                                }

                                #[inline]
                                fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self, rapira::RapiraError>
                                where
                                    Self: Sized,
                                {
                                    let val: u8 = u8::from_slice(slice)?;
                                    let t = <#primitive_name as core::convert::TryFrom<u8>>::try_from(val).map_err(|_| rapira::RapiraError::EnumVariantError)?;
                                    match t {
                                        #(#from_slice_unchecked)*
                                    }
                                }

                                #[inline]
                                unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self, rapira::RapiraError>
                                where
                                    Self: Sized,
                                {
                                    let val: u8 = u8::from_slice_unsafe(slice)?;
                                    let t = <#primitive_name as primitive_enum::UnsafeFromU8>::from_unsafe(val);
                                    match t {
                                        #(#from_slice_unsafe)*
                                    }
                                }

                                #[inline]
                                fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<(), rapira::RapiraError> {
                                    let t = self.get_primitive_enum() as u8;
                                    rapira::push(slice, cursor, t);
                                    match self {
                                        #(#try_convert_to_bytes)*
                                    }
                                    Ok(())
                                }

                                #[inline]
                                fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
                                    let t = self.get_primitive_enum() as u8;
                                    rapira::push(slice, cursor, t);
                                    match self {
                                        #(#convert_to_bytes)*
                                    }
                                }

                                #[inline]
                                fn size(&self) -> usize {
                                    1 + match self {
                                        #(#size)*
                                    }
                                }
                            }
                        };

                        proc_macro::TokenStream::from(gen)
                    }
                    None => {
                        let mut enum_sizes: Vec<TokenStream> = Vec::with_capacity(variants_len);
                        let mut size: Vec<TokenStream> = Vec::with_capacity(variants_len);
                        let mut check_bytes: Vec<TokenStream> = Vec::with_capacity(variants_len);
                        let mut from_slice: Vec<TokenStream> = Vec::with_capacity(variants_len);
                        let mut from_slice_unchecked: Vec<TokenStream> =
                            Vec::with_capacity(variants_len);
                        let mut from_slice_unsafe: Vec<TokenStream> =
                            Vec::with_capacity(variants_len);
                        let mut try_convert_to_bytes: Vec<TokenStream> =
                            Vec::with_capacity(variants_len);
                        let mut convert_to_bytes: Vec<TokenStream> =
                            Vec::with_capacity(variants_len);

                        for (variant_id, variant) in
                            data_enum.variants.iter().enumerate().map(|(idx, variant)| {
                                let id: u8 = variant
                                    .attrs
                                    .iter()
                                    .find_map(|a| {
                                        a.path.segments.first().and_then(|segment| {
                                            if segment.ident != "idx" {
                                                return None;
                                            }
                                            match a.parse_args::<Meta>() {
                                                Ok(Meta::List(list)) => {
                                                    let a = list.nested.first().unwrap();
                                                    let int: u8 = match a {
                                                        NestedMeta::Lit(Lit::Int(i)) => {
                                                            i.base10_parse::<u8>().unwrap()
                                                        }
                                                        _ => {
                                                            panic!("error meta type")
                                                        }
                                                    };
                                                    Some(int)
                                                }
                                                Ok(_) => None,
                                                Err(_) => None,
                                            }
                                        })
                                    })
                                    .unwrap_or(idx as u8);
                                (id, variant)
                            })
                        {
                            let variant_name = &variant.ident;

                            match &variant.fields {
                                Fields::Unit => {
                                    from_slice.push(quote! {
                                        #variant_id => {
                                            Ok(#name::#variant_name)
                                        }
                                    });
                                    check_bytes.push(quote! {
                                        #variant_id => {}
                                    });
                                    from_slice_unchecked.push(quote! {
                                        #variant_id => {
                                            Ok(#name::#variant_name)
                                        }
                                    });
                                    from_slice_unsafe.push(quote! {
                                        #variant_id => {
                                            Ok(#name::#variant_name)
                                        }
                                    });
                                    try_convert_to_bytes.push(quote! {
                                        #name::#variant_name => {}
                                    });
                                    convert_to_bytes.push(quote! {
                                        #name::#variant_name => {}
                                    });
                                    size.push(quote! {
                                        #name::#variant_name => 0,
                                    });
                                    enum_sizes.push(quote! {
                                        None,
                                    });
                                }
                                Fields::Unnamed(fields) => {
                                    let len = fields.unnamed.len();
                                    let fields = &fields.unnamed;

                                    let mut field_names: Vec<TokenStream> = Vec::with_capacity(len);
                                    let mut fields_static_sizes: Vec<TokenStream> =
                                        Vec::with_capacity(len);
                                    let mut fields_size: Vec<TokenStream> = Vec::with_capacity(len);
                                    let mut fields_from_slice: Vec<TokenStream> =
                                        Vec::with_capacity(len);
                                    let mut fields_check_bytes: Vec<TokenStream> =
                                        Vec::with_capacity(len);
                                    let mut fields_from_slice_unchecked: Vec<TokenStream> =
                                        Vec::with_capacity(len);
                                    let mut fields_from_slice_unsafe: Vec<TokenStream> =
                                        Vec::with_capacity(len);
                                    let mut fields_try_convert_to_bytes: Vec<TokenStream> =
                                        Vec::with_capacity(len);
                                    let mut fields_convert_to_bytes: Vec<TokenStream> =
                                        Vec::with_capacity(len);

                                    for (idx, field) in fields.iter().enumerate() {
                                        let typ = &field.ty;
                                        let field_name = syn::Ident::new(
                                            &format!("arg{}", idx),
                                            Span::call_site(),
                                        );

                                        field_names.push(quote! { #field_name, });
                                        fields_static_sizes.push(quote! {
                                            <#typ>::STATIC_SIZE,
                                        });
                                        fields_size.push(quote! { + (match <#typ>::STATIC_SIZE {
                                            Some(s) => s,
                                            None => #field_name.size()
                                        }) });
                                        fields_from_slice.push(quote! {
                                            let #field_name = <#typ>::from_slice(slice)?;
                                        });
                                        fields_check_bytes.push(quote! {
                                            <#typ>::check_bytes(slice)?;
                                        });
                                        fields_from_slice_unchecked.push(quote! {
                                            let #field_name = <#typ>::from_slice_unchecked(slice)?;
                                        });
                                        fields_from_slice_unsafe.push(quote! {
                                            let #field_name = <#typ>::from_slice_unsafe(slice)?;
                                        });
                                        fields_try_convert_to_bytes.push(quote! {
                                            #field_name.try_convert_to_bytes(slice, cursor)?;
                                        });
                                        fields_convert_to_bytes.push(quote! {
                                            #field_name.convert_to_bytes(slice, cursor);
                                        });
                                    }

                                    size.push(quote! {
                                        #name::#variant_name(#(#field_names)*) => {
                                            0 #(#fields_size)*
                                        },
                                    });
                                    enum_sizes.push(quote! {
                                        rapira::static_size([#(#fields_static_sizes)*]),
                                    });
                                    from_slice.push(quote! {
                                        #variant_id => {
                                            #(#fields_from_slice)*
                                            Ok(#name::#variant_name(#(#field_names)*))
                                        }
                                    });
                                    check_bytes.push(quote! {
                                        #variant_id => {
                                            #(#fields_check_bytes)*
                                        }
                                    });
                                    from_slice_unchecked.push(quote! {
                                        #variant_id => {
                                            #(#fields_from_slice_unchecked)*
                                            Ok(#name::#variant_name(#(#field_names)*))
                                        }
                                    });

                                    from_slice_unsafe.push(quote! {
                                        #variant_id => {
                                            #(#fields_from_slice_unsafe)*
                                            Ok(#name::#variant_name(#(#field_names)*))
                                        }
                                    });

                                    try_convert_to_bytes.push(quote! {
                                        #name::#variant_name(#(#field_names)*) => {
                                            rapira::push(slice, cursor, #variant_id);
                                            #(#fields_try_convert_to_bytes)*
                                        }
                                    });

                                    convert_to_bytes.push(quote! {
                                        #name::#variant_name(#(#field_names)*) => {
                                            rapira::push(slice, cursor, #variant_id);
                                            #(#fields_convert_to_bytes)*
                                        }
                                    });
                                }
                                Fields::Named(fields) => {
                                    let len = fields.named.len();
                                    let fields = &fields.named;

                                    let mut fields_insert: Vec<(Field, u32)> =
                                        Vec::with_capacity(len);
                                    let mut seq = 0u32;

                                    for field in fields.iter() {
                                        let field_idx = field
                                            .attrs
                                            .iter()
                                            .find_map(|a| {
                                                a.path.segments.first().and_then(|segment| {
                                                    if segment.ident != "idx" {
                                                        return None;
                                                    }
                                                    match a.parse_args::<Meta>() {
                                                        Ok(Meta::List(list)) => {
                                                            let a = list.nested.first().unwrap();
                                                            let int: u32 = match a {
                                                                NestedMeta::Lit(Lit::Int(i)) => {
                                                                    i.base10_parse::<u32>().unwrap()
                                                                }
                                                                _ => {
                                                                    panic!("error meta type")
                                                                }
                                                            };
                                                            Some(int)
                                                        }
                                                        Ok(_) => None,
                                                        Err(_) => None,
                                                    }
                                                })
                                            })
                                            .unwrap_or_else(|| {
                                                let current_seq = seq;
                                                seq += 1;
                                                current_seq
                                            });

                                        fields_insert.push((field.clone(), field_idx));
                                    }

                                    fields_insert
                                        .sort_by(|(_, idx_a), (_, idx_b)| idx_a.cmp(idx_b));

                                    let mut field_names: Vec<TokenStream> = Vec::with_capacity(len);
                                    let mut fields_from_slice: Vec<TokenStream> =
                                        Vec::with_capacity(len);
                                    let mut fields_check_bytes: Vec<TokenStream> =
                                        Vec::with_capacity(len);
                                    let mut fields_from_slice_unchecked: Vec<TokenStream> =
                                        Vec::with_capacity(len);
                                    let mut fields_from_slice_unsafe: Vec<TokenStream> =
                                        Vec::with_capacity(len);
                                    let mut fields_try_convert_to_bytes: Vec<TokenStream> =
                                        Vec::with_capacity(len);
                                    let mut fields_convert_to_bytes: Vec<TokenStream> =
                                        Vec::with_capacity(len);
                                    let mut fields_size: Vec<TokenStream> = Vec::with_capacity(len);
                                    let mut fields_static_sizes: Vec<TokenStream> =
                                        Vec::with_capacity(len);

                                    for field in fields_insert.iter().map(|(f, _)| f) {
                                        let typ = &field.ty;
                                        let field_name = field.ident.as_ref().unwrap();

                                        fields_from_slice.push(quote! {
                                            let #field_name = <#typ>::from_slice(slice)?;
                                        });
                                        fields_check_bytes.push(quote! {
                                            <#typ>::check_bytes(slice)?;
                                        });
                                        fields_from_slice_unchecked.push(quote! {
                                            let #field_name = <#typ>::from_slice_unchecked(slice)?;
                                        });
                                        fields_from_slice_unsafe.push(quote! {
                                            let #field_name = <#typ>::from_slice_unsafe(slice)?;
                                        });
                                        fields_try_convert_to_bytes.push(quote! {
                                            #field_name.try_convert_to_bytes(slice, cursor)?;
                                        });
                                        fields_convert_to_bytes.push(quote! {
                                            #field_name.convert_to_bytes(slice, cursor);
                                        });
                                        fields_size.push(quote! { + (match <#typ>::STATIC_SIZE {
                                            Some(s) => s,
                                            None => #field_name.size()
                                        }) });
                                        fields_static_sizes.push(quote! {
                                            <#typ>::STATIC_SIZE,
                                        });
                                        field_names.push(quote! { #field_name, });
                                    }

                                    size.push(quote! {
                                        #name::#variant_name{#(#field_names)*} => {
                                            0 #(#fields_size)*
                                        },
                                    });
                                    enum_sizes.push(quote! {
                                        rapira::static_size([#(#fields_static_sizes)*]),
                                    });
                                    from_slice.push(quote! {
                                        #variant_id => {
                                            #(#fields_from_slice)*
                                            Ok(#name::#variant_name{#(#field_names)*})
                                        }
                                    });
                                    check_bytes.push(quote! {
                                        #variant_id => {
                                            #(#fields_check_bytes)*
                                        }
                                    });
                                    from_slice_unchecked.push(quote! {
                                        #variant_id => {
                                            #(#fields_from_slice_unchecked)*
                                            Ok(#name::#variant_name{#(#field_names)*})
                                        }
                                    });

                                    from_slice_unsafe.push(quote! {
                                        #variant_id => {
                                            #(#fields_from_slice_unsafe)*
                                            Ok(#name::#variant_name{#(#field_names)*})
                                        }
                                    });

                                    try_convert_to_bytes.push(quote! {
                                        #name::#variant_name{#(#field_names)*} => {
                                            rapira::push(slice, cursor, #variant_id);
                                            #(#fields_try_convert_to_bytes)*
                                        }
                                    });

                                    convert_to_bytes.push(quote! {
                                        #name::#variant_name{#(#field_names)*} => {
                                            rapira::push(slice, cursor, #variant_id);
                                            #(#fields_convert_to_bytes)*
                                        }
                                    });
                                }
                            }
                        }

                        let gen = quote! {
                            impl rapira::Rapira for #name {
                                const STATIC_SIZE: Option<usize> = rapira::enum_size([#(#enum_sizes)*]);

                                #[inline]
                                fn from_slice(slice: &mut &[u8]) -> Result<Self, rapira::RapiraError>
                                where
                                    Self: Sized,
                                {
                                    let val: u8 = u8::from_slice(slice)?;
                                    match val {
                                        #(#from_slice)*
                                        _ => rapira::RapiraError::EnumVariantError,
                                    }
                                }

                                #[inline]
                                fn check_bytes(slice: &mut &[u8]) -> Result<(), rapira::RapiraError>
                                where
                                    Self: Sized,
                                {
                                    let val: u8 = u8::from_slice(slice)?;
                                    match val {
                                        #(#check_bytes)*
                                        _ => rapira::RapiraError::EnumVariantError,
                                    }
                                    Ok(())
                                }

                                #[inline]
                                fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self, rapira::RapiraError>
                                where
                                    Self: Sized,
                                {
                                    let val: u8 = u8::from_slice(slice)?;
                                    match val {
                                        #(#from_slice_unchecked)*
                                        _ => rapira::RapiraError::EnumVariantError,
                                    }
                                }

                                #[inline]
                                unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self, rapira::RapiraError>
                                where
                                    Self: Sized,
                                {
                                    let val: u8 = u8::from_slice_unsafe(slice)?;
                                    match val {
                                        #(#from_slice_unsafe)*
                                        _ => rapira::RapiraError::EnumVariantError,
                                    }
                                }

                                #[inline]
                                fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<(), rapira::RapiraError> {
                                    match self {
                                        #(#try_convert_to_bytes)*
                                    }
                                    Ok(())
                                }

                                #[inline]
                                fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
                                    match self {
                                        #(#convert_to_bytes)*
                                    }
                                }

                                #[inline]
                                fn size(&self) -> usize {
                                    1 + match self {
                                        #(#size)*
                                    }
                                }
                            }
                        };

                        proc_macro::TokenStream::from(gen)
                    }
                }
            }
        }
        Data::Union(_) => {
            panic!("unions not supported, but Rust enums is implemented GetType trait (use Enums instead)")
        }
    }
}
