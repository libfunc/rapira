extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{DataStruct, ExprPath, Field, Fields, Generics, LitInt};

use crate::{
    field_attrs::{extract_idx_attr, extract_with_attr, skip_attr},
    shared::build_ident,
};

pub fn struct_serializer(
    data_struct: &DataStruct,
    name: &Ident,
    generics: Generics,
    is_debug: bool,
) -> proc_macro::TokenStream {
    let fields = &data_struct.fields;
    match fields {
        Fields::Named(fields) => {
            let named = &fields.named;
            let named_len = named.len();

            let mut fields_insert: Vec<(Field, u32, Option<ExprPath>)> =
                Vec::with_capacity(named_len);
            let mut seq = 0u32;

            let iter = named.iter().filter(|field| !skip_attr(&field.attrs));

            for field in iter {
                let field_idx = extract_idx_attr(&field.attrs).unwrap_or_else(|| {
                    let current_seq = seq;
                    seq += 1;
                    current_seq
                });

                let field_with_attr = extract_with_attr(&field.attrs);

                fields_insert.push((field.clone(), field_idx, field_with_attr));
            }

            fields_insert.sort_by(|(_, idx_a, _), (_, idx_b, _)| idx_a.cmp(idx_b));

            let mut field_names: Vec<TokenStream> = Vec::with_capacity(named_len);
            let mut from_slice: Vec<TokenStream> = Vec::with_capacity(named_len);
            let mut debug_from_slice: Vec<TokenStream> = Vec::with_capacity(named_len);
            let mut check_bytes: Vec<TokenStream> = Vec::with_capacity(named_len);
            let mut from_slice_unchecked: Vec<TokenStream> = Vec::with_capacity(named_len);
            let mut from_slice_unsafe: Vec<TokenStream> = Vec::with_capacity(named_len);
            let mut try_convert_to_bytes: Vec<TokenStream> = Vec::with_capacity(named_len);
            let mut convert_to_bytes: Vec<TokenStream> = Vec::with_capacity(named_len);
            let mut size: Vec<TokenStream> = Vec::with_capacity(named_len);
            let mut static_sizes: Vec<TokenStream> = Vec::with_capacity(named_len);
            let mut min_size: Vec<TokenStream> = Vec::with_capacity(named_len);

            for (field, _, with_attr) in fields_insert.iter() {
                let ident = field.ident.as_ref().unwrap();
                let typ = &field.ty;

                field_names.push(quote! { #ident, });

                match with_attr {
                    Some(with_attr) => {
                        static_sizes.push(quote! {
                            #with_attr::static_size(core::marker::PhantomData::<#typ>),
                        });
                        min_size.push(quote! {
                            #with_attr::min_size(core::marker::PhantomData::<#typ>),
                        });
                        size.push(
                            quote! { + (match #with_attr::static_size(core::marker::PhantomData::<#typ>) {
                                Some(s) => s,
                                None => #with_attr::size(&self.#ident)
                            }) },
                        );
                        check_bytes.push(quote! {
                            #with_attr::check_bytes(core::marker::PhantomData::<#typ>, slice)?;
                        });
                        from_slice.push(quote! {
                            let #ident: #typ = #with_attr::from_slice(slice)?;
                        });
                        debug_from_slice.push(quote! {
                            let len = slice.len();
                            println!("Field: {}, Type: {}", stringify!(#ident), stringify!(#typ));
                            let res = #with_attr::from_slice(slice).inspect(|v| {
                                println!("len: {len}, {}: {v:?}", stringify!(#ident));
                            }).inspect_err(|err| {
                                println!("len: {len}, err: {err:?}");
                            });
                            let #ident: #typ = res?;
                        });
                        from_slice_unchecked.push(quote! {
                            let #ident: #typ = #with_attr::from_slice_unchecked(slice)?;
                        });
                        from_slice_unsafe.push(quote! {
                            let #ident: #typ = #with_attr::from_slice_unsafe(slice)?;
                        });
                        try_convert_to_bytes.push(quote! {
                            #with_attr::try_convert_to_bytes(&self.#ident, slice, cursor)?;
                        });
                        convert_to_bytes.push(quote! {
                            #with_attr::convert_to_bytes(&self.#ident, slice, cursor);
                        });
                    }
                    None => {
                        from_slice.push(quote! {
                            let #ident = <#typ as rapira::Rapira>::from_slice(slice)?;
                        });
                        debug_from_slice.push(quote! {
                            let len = slice.len();
                            println!("Field: {}, Type: {}", stringify!(#ident), stringify!(#typ));
                            let res = <#typ as rapira::Rapira>::from_slice(slice).inspect(|v| {
                                println!("len: {len}, {}: {v:?}", stringify!(#ident));
                            }).inspect_err(|err| {
                                println!("len: {len}, err: {err:?}");
                            });
                            let #ident = res?;
                        });
                        check_bytes.push(quote! {
                            <#typ as rapira::Rapira>::check_bytes(slice)?;
                        });
                        from_slice_unchecked.push(quote! {
                            let #ident = <#typ as rapira::Rapira>::from_slice_unchecked(slice)?;
                        });
                        from_slice_unsafe.push(quote! {
                            let #ident = <#typ as rapira::Rapira>::from_slice_unsafe(slice)?;
                        });
                        try_convert_to_bytes.push(quote! {
                            self.#ident.try_convert_to_bytes(slice, cursor)?;
                        });
                        convert_to_bytes.push(quote! {
                            self.#ident.convert_to_bytes(slice, cursor);
                        });
                        size.push(quote! { + (match <#typ as rapira::Rapira>::STATIC_SIZE {
                            Some(s) => s,
                            None => self.#ident.size()
                        }) });
                        static_sizes.push(quote! {
                            <#typ as rapira::Rapira>::STATIC_SIZE,
                        });
                        min_size.push(quote! {
                            <#typ as rapira::Rapira>::MIN_SIZE,
                        });
                    }
                }
            }

            let name_with_generics = build_ident(name, generics);

            let debug_parse = if is_debug {
                quote! {
                    /// Deserializes a value from a byte slice with debug logging.
                    /// This method logs the struct name, field names, types, and values during deserialization.
                    /// Useful for debugging serialization/deserialization issues.
                    #[inline]
                    fn debug_from_slice(slice: &mut &[u8]) -> rapira::Result<Self>
                    where
                        Self: Sized + std::fmt::Debug,
                    {
                        println!("Struct: {}", stringify!(#name));
                        #(#debug_from_slice)*
                        Ok(#name {
                            #(#field_names)*
                        })
                    }
                }
            } else {
                quote!()
            };

            let res = quote! {
                #name_with_generics {
                    const STATIC_SIZE: Option<usize> = rapira::static_size([#(#static_sizes)*]);
                    const MIN_SIZE: usize = rapira::min_size(&[#(#min_size)*]);

                    #[inline]
                    fn from_slice(slice: &mut &[u8]) -> rapira::Result<Self>
                    where
                        Self: Sized,
                    {
                        #(#from_slice)*
                        Ok(#name {
                            #(#field_names)*
                        })
                    }

                    #debug_parse

                    #[inline]
                    fn check_bytes(slice: &mut &[u8]) -> rapira::Result<()>
                    where
                        Self: Sized,
                    {
                        #(#check_bytes)*
                        Ok(())
                    }

                    #[inline]
                    unsafe fn from_slice_unchecked(slice: &mut &[u8]) -> rapira::Result<Self>
                    where
                        Self: Sized,
                    {
                        #(#from_slice_unchecked)*
                        Ok(#name {
                            #(#field_names)*
                        })
                    }

                    #[inline]
                    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> rapira::Result<Self>
                    where
                        Self: Sized,
                    {
                        #(#from_slice_unsafe)*
                        Ok(#name {
                            #(#field_names)*
                        })
                    }

                    #[inline]
                    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> rapira::Result<()> {
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
            proc_macro::TokenStream::from(res)
        }
        Fields::Unnamed(fields) => {
            let unnamed = &fields.unnamed;
            let unnamed_len = unnamed.len();
            let mut field_names: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
            let mut from_slice: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
            let mut debug_from_slice: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
            let mut check_bytes: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
            let mut from_slice_unchecked: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
            let mut from_slice_unsafe: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
            let mut try_convert_to_bytes: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
            let mut convert_to_bytes: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
            let mut size: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
            let mut static_sizes: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
            let mut min_size: Vec<TokenStream> = Vec::with_capacity(unnamed_len);

            for (idx, field) in unnamed.iter().enumerate() {
                let id = syn::Lit::Int(LitInt::new(&idx.to_string(), Span::call_site()));
                let typ = &field.ty;
                let field_name = syn::Ident::new(&format!("arg{idx}"), Span::call_site());
                let field_name_into = quote! { #field_name, };
                let with_attr = extract_with_attr(&field.attrs);

                field_names.push(field_name_into);

                match with_attr {
                    Some(with_attr) => {
                        static_sizes.push(quote! {
                            #with_attr::static_size(core::marker::PhantomData::<#typ>),
                        });
                        min_size.push(quote! {
                            #with_attr::min_size(core::marker::PhantomData::<#typ>),
                        });
                        size.push(
                            quote! { + (match #with_attr::static_size(core::marker::PhantomData::<#typ>) {
                                Some(s) => s,
                                None => #with_attr::size(&self.#id)
                            }) },
                        );
                        check_bytes.push(quote! {
                            #with_attr::check_bytes(core::marker::PhantomData::<#typ>, slice)?;
                        });
                        from_slice.push(quote! {
                            let #field_name: #typ = #with_attr::from_slice(slice)?;
                        });
                        debug_from_slice.push(quote! {
                            let len = slice.len();
                            println!("Field: unnamed (index {}), Type: {}", #idx, stringify!(#typ));
                            let res = #with_attr::from_slice(slice).inspect(|v| {
                                println!("len: {len}, unnamed (index {}): {v:?}", #idx);
                            }).inspect_err(|err| {
                                println!("len: {len}, err: {err:?}");
                            });
                            let #field_name: #typ = res?;
                        });
                        from_slice_unchecked.push(quote! {
                            let #field_name: #typ = #with_attr::from_slice_unchecked(slice)?;
                        });
                        from_slice_unsafe.push(quote! {
                            let #field_name: #typ = #with_attr::from_slice_unsafe(slice)?;
                        });
                        try_convert_to_bytes.push(quote! {
                            #with_attr::try_convert_to_bytes(&self.#id, slice, cursor)?;
                        });
                        convert_to_bytes.push(quote! {
                            #with_attr::convert_to_bytes(&self.#id, slice, cursor);
                        });
                    }
                    None => {
                        from_slice.push(quote! {
                            let #field_name = <#typ as rapira::Rapira>::from_slice(slice)?;
                        });
                        debug_from_slice.push(quote! {
                            let len = slice.len();
                            println!("Field: unnamed (index {}), Type: {}", #idx, stringify!(#typ));
                            let res = <#typ as rapira::Rapira>::from_slice(slice).inspect(|v| {
                                println!("len: {len}, unnamed (index {}): {v:?}", #idx);
                            }).inspect_err(|err| {
                                println!("len: {len}, err: {err:?}");
                            });
                            let #field_name = res?;
                        });
                        check_bytes.push(quote! {
                            <#typ as rapira::Rapira>::check_bytes(slice)?;
                        });
                        from_slice_unchecked.push(quote! {
                            let #field_name = <#typ as rapira::Rapira>::from_slice_unchecked(slice)?;
                        });
                        from_slice_unsafe.push(quote! {
                            let #field_name = <#typ as rapira::Rapira>::from_slice_unsafe(slice)?;
                        });
                        try_convert_to_bytes.push(quote! {
                            self.#id.try_convert_to_bytes(slice, cursor)?;
                        });
                        convert_to_bytes.push(quote! {
                            self.#id.convert_to_bytes(slice, cursor);
                        });
                        size.push(quote! { + (match <#typ as rapira::Rapira>::STATIC_SIZE {
                            Some(s) => s,
                            None => self.#id.size()
                        }) });
                        static_sizes.push(quote! {
                            <#typ as rapira::Rapira>::STATIC_SIZE,
                        });
                        min_size.push(quote! {
                            <#typ as rapira::Rapira>::MIN_SIZE,
                        });
                    }
                }
            }

            let name_with_generics = build_ident(name, generics);

            let debug_parse = if is_debug {
                quote! {
                    /// Deserializes a value from a byte slice with debug logging.
                    /// This method logs the struct name, field indices, types, and values during deserialization.
                    /// Useful for debugging serialization/deserialization issues.
                    #[inline]
                    fn debug_from_slice(slice: &mut &[u8]) -> rapira::Result<Self>
                    where
                        Self: Sized + std::fmt::Debug,
                    {
                        println!("Struct: {}", stringify!(#name));
                        #(#debug_from_slice)*
                        Ok(#name(#(#field_names)*))
                    }
                }
            } else {
                quote!()
            };

            let res = quote! {
                #name_with_generics {
                    const STATIC_SIZE: Option<usize> = rapira::static_size([#(#static_sizes)*]);
                    const MIN_SIZE: usize = rapira::min_size(&[#(#min_size)*]);

                    #[inline]
                    fn from_slice(slice: &mut &[u8]) -> rapira::Result<Self>
                    where
                        Self: Sized,
                    {
                        #(#from_slice)*
                        Ok(#name(#(#field_names)*))
                    }

                    #debug_parse

                    #[inline]
                    fn check_bytes(slice: &mut &[u8]) -> rapira::Result<()>
                    where
                        Self: Sized,
                    {
                        #(#check_bytes)*
                        Ok(())
                    }

                    #[inline]
                    unsafe fn from_slice_unchecked(slice: &mut &[u8]) -> rapira::Result<Self>
                    where
                        Self: Sized,
                    {
                        #(#from_slice_unchecked)*
                        Ok(#name(#(#field_names)*))
                    }

                    #[inline]
                    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> rapira::Result<Self>
                    where
                        Self: Sized,
                    {
                        #(#from_slice_unsafe)*
                        Ok(#name(#(#field_names)*))
                    }

                    #[inline]
                    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> rapira::Result<()> {
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

            proc_macro::TokenStream::from(res)
        }
        Fields::Unit => proc_macro::TokenStream::from(quote! {
            impl rapira::Rapira for #name {
                const STATIC_SIZE: Option<usize> = Some(0);
                const MIN_SIZE: usize = 0;

                #[inline]
                fn from_slice(slice: &mut &[u8]) -> rapira::Result<Self>
                where
                    Self: Sized,
                {
                    Ok(#name)
                }

                #[inline]
                fn check_bytes(slice: &mut &[u8]) -> rapira::Result<()>
                where
                    Self: Sized,
                {
                    Ok(())
                }

                #[inline]
                unsafe fn from_slice_unchecked(slice: &mut &[u8]) -> rapira::Result<Self>
                where
                    Self: Sized,
                {
                    Ok(#name)
                }

                #[inline]
                unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> rapira::Result<Self>
                where
                    Self: Sized,
                {
                    Ok(#name)
                }

                #[inline]
                fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> rapira::Result<()> {
                    Ok(())
                }

                #[inline]
                fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {}

                #[inline]
                fn size(&self) -> usize { 0 }
            }
        }),
    }
}
