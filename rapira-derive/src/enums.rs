extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{DataEnum, Expr, Field, Fields, Generics};

use crate::{
    field_attrs::{extract_idx_attr, extract_with_attr},
    shared::build_ident,
};

pub fn enum_serializer(
    data_enum: &DataEnum,
    name: &Ident,
    static_size: Option<Expr>,
    min_size: Option<Expr>,
    generics: Generics,
) -> proc_macro::TokenStream {
    let variants_len = data_enum.variants.len();

    let mut enum_sizes: Vec<TokenStream> = Vec::with_capacity(variants_len);
    let mut min_sizes: Vec<TokenStream> = Vec::with_capacity(variants_len);
    let mut size: Vec<TokenStream> = Vec::with_capacity(variants_len);
    let mut check_bytes: Vec<TokenStream> = Vec::with_capacity(variants_len);
    let mut from_slice: Vec<TokenStream> = Vec::with_capacity(variants_len);
    let mut from_slice_unchecked: Vec<TokenStream> = Vec::with_capacity(variants_len);
    let mut from_slice_unsafe: Vec<TokenStream> = Vec::with_capacity(variants_len);
    let mut try_convert_to_bytes: Vec<TokenStream> = Vec::with_capacity(variants_len);
    let mut convert_to_bytes: Vec<TokenStream> = Vec::with_capacity(variants_len);

    let variants_iter = data_enum.variants.iter().enumerate().map(|(idx, variant)| {
        let id: u8 = extract_idx_attr(&variant.attrs)
            .map(|idx| idx as u8)
            .unwrap_or(idx as u8);

        (id, variant)
    });

    for (variant_id, variant) in variants_iter {
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
                    #name::#variant_name => {
                        rapira::push(slice, cursor, #variant_id);
                    }
                });
                convert_to_bytes.push(quote! {
                    #name::#variant_name => {
                        rapira::push(slice, cursor, #variant_id);
                    }
                });
                size.push(quote! {
                    #name::#variant_name => 0,
                });
                enum_sizes.push(quote! {
                    None,
                });
                min_sizes.push(quote! {
                    0,
                });
            }
            Fields::Unnamed(fields) => {
                let len = fields.unnamed.len();
                let fields = &fields.unnamed;

                let mut field_names: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_static_sizes: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_min_sizes: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_size: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_from_slice: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_check_bytes: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_from_slice_unchecked: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_from_slice_unsafe: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_try_convert_to_bytes: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_convert_to_bytes: Vec<TokenStream> = Vec::with_capacity(len);

                for (idx, field) in fields.iter().enumerate() {
                    let typ = &field.ty;
                    let field_name = syn::Ident::new(&format!("arg{idx}"), Span::call_site());
                    let with_attr = extract_with_attr(&field.attrs);

                    field_names.push(quote! { #field_name, });

                    match with_attr {
                        Some(with_attr) => {
                            fields_static_sizes.push(quote! {
                                #with_attr::static_size(core::marker::PhantomData::<#typ>),
                            });
                            fields_min_sizes.push(quote! {
                                #with_attr::min_size(core::marker::PhantomData::<#typ>),
                            });
                            fields_size.push(
                                quote! { + (match #with_attr::static_size(core::marker::PhantomData::<#typ>) {
                                    Some(s) => s,
                                    None => #with_attr::size(#field_name)
                                }) },
                            );
                            fields_check_bytes.push(quote! {
                                #with_attr::check_bytes(core::marker::PhantomData::<#typ>, slice)?;
                            });
                            fields_from_slice.push(quote! {
                                let #field_name: #typ = #with_attr::from_slice(slice)?;
                            });
                            fields_from_slice_unchecked.push(quote! {
                                let #field_name: #typ = #with_attr::from_slice_unchecked(slice)?;
                            });
                            fields_from_slice_unsafe.push(quote! {
                                let #field_name: #typ = #with_attr::from_slice_unsafe(slice)?;
                            });
                            fields_try_convert_to_bytes.push(quote! {
                                #with_attr::try_convert_to_bytes(#field_name, slice, cursor)?;
                            });
                            fields_convert_to_bytes.push(quote! {
                                #with_attr::convert_to_bytes(#field_name, slice, cursor);
                            });
                        }
                        None => {
                            fields_static_sizes.push(quote! {
                                <#typ as rapira::Rapira>::STATIC_SIZE,
                            });
                            fields_min_sizes.push(quote! {
                                <#typ as rapira::Rapira>::MIN_SIZE,
                            });
                            fields_size.push(
                                quote! { + (match <#typ as rapira::Rapira>::STATIC_SIZE {
                                    Some(s) => s,
                                    None => #field_name.size()
                                }) },
                            );
                            fields_from_slice.push(quote! {
                                let #field_name = <#typ as rapira::Rapira>::from_slice(slice)?;
                            });
                            fields_check_bytes.push(quote! {
                                <#typ as rapira::Rapira>::check_bytes(slice)?;
                            });
                            fields_from_slice_unchecked.push(quote! {
                                let #field_name = <#typ as rapira::Rapira>::from_slice_unchecked(slice)?;
                            });
                            fields_from_slice_unsafe.push(quote! {
                                let #field_name = <#typ as rapira::Rapira>::from_slice_unsafe(slice)?;
                            });
                            fields_try_convert_to_bytes.push(quote! {
                                #field_name.try_convert_to_bytes(slice, cursor)?;
                            });
                            fields_convert_to_bytes.push(quote! {
                                #field_name.convert_to_bytes(slice, cursor);
                            });
                        }
                    }
                }

                size.push(quote! {
                    #name::#variant_name(#(#field_names)*) => {
                        0 #(#fields_size)*
                    },
                });
                enum_sizes.push(quote! {
                    rapira::static_size([#(#fields_static_sizes)*]),
                });
                min_sizes.push(quote! {
                    rapira::min_size(&[#(#fields_min_sizes)*]),
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

                let mut fields_insert: Vec<(Field, u32)> = Vec::with_capacity(len);
                let mut seq = 0u32;

                for field in fields.iter() {
                    let field_idx = extract_idx_attr(&field.attrs);
                    let field_idx = field_idx.unwrap_or_else(|| {
                        let current_seq = seq;
                        seq += 1;
                        current_seq
                    });

                    fields_insert.push((field.clone(), field_idx));
                }

                fields_insert.sort_by(|(_, idx_a), (_, idx_b)| idx_a.cmp(idx_b));

                let mut field_names: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_from_slice: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_check_bytes: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_from_slice_unchecked: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_from_slice_unsafe: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_try_convert_to_bytes: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_convert_to_bytes: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_size: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_static_sizes: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_min_sizes: Vec<TokenStream> = Vec::with_capacity(len);

                for field in fields_insert.iter().map(|(f, _)| f) {
                    let typ = &field.ty;
                    let field_name = field.ident.as_ref().unwrap();
                    let with_attr = extract_with_attr(&field.attrs);

                    field_names.push(quote! { #field_name, });

                    match with_attr {
                        Some(with_attr) => {
                            fields_static_sizes.push(quote! {
                                #with_attr::static_size(core::marker::PhantomData::<#typ>),
                            });
                            fields_min_sizes.push(quote! {
                                #with_attr::min_size(core::marker::PhantomData::<#typ>),
                            });
                            fields_size.push(
                                quote! { + (match #with_attr::static_size(core::marker::PhantomData::<#typ>) {
                                    Some(s) => s,
                                    None => #with_attr::size(#field_name)
                                }) },
                            );
                            fields_check_bytes.push(quote! {
                                #with_attr::check_bytes(core::marker::PhantomData::<#typ>, slice)?;
                            });
                            fields_from_slice.push(quote! {
                                let #field_name: #typ = #with_attr::from_slice(slice)?;
                            });
                            fields_from_slice_unchecked.push(quote! {
                                let #field_name: #typ = #with_attr::from_slice_unchecked(slice)?;
                            });
                            fields_from_slice_unsafe.push(quote! {
                                let #field_name: #typ = #with_attr::from_slice_unsafe(slice)?;
                            });
                            fields_try_convert_to_bytes.push(quote! {
                                #with_attr::try_convert_to_bytes(#field_name, slice, cursor)?;
                            });
                            fields_convert_to_bytes.push(quote! {
                                #with_attr::convert_to_bytes(#field_name, slice, cursor);
                            });
                        }
                        None => {
                            fields_from_slice.push(quote! {
                                let #field_name = <#typ as rapira::Rapira>::from_slice(slice)?;
                            });
                            fields_check_bytes.push(quote! {
                                <#typ as rapira::Rapira>::check_bytes(slice)?;
                            });
                            fields_from_slice_unchecked.push(quote! {
                                let #field_name = <#typ as rapira::Rapira>::from_slice_unchecked(slice)?;
                            });
                            fields_from_slice_unsafe.push(quote! {
                                let #field_name = <#typ as rapira::Rapira>::from_slice_unsafe(slice)?;
                            });
                            fields_try_convert_to_bytes.push(quote! {
                                #field_name.try_convert_to_bytes(slice, cursor)?;
                            });
                            fields_convert_to_bytes.push(quote! {
                                #field_name.convert_to_bytes(slice, cursor);
                            });
                            fields_size.push(
                                quote! { + (match <#typ as rapira::Rapira>::STATIC_SIZE {
                                    Some(s) => s,
                                    None => #field_name.size()
                                }) },
                            );
                            fields_static_sizes.push(quote! {
                                <#typ as rapira::Rapira>::STATIC_SIZE,
                            });
                            fields_min_sizes.push(quote! {
                                <#typ as rapira::Rapira>::MIN_SIZE,
                            });
                        }
                    }
                }

                size.push(quote! {
                    #name::#variant_name{#(#field_names)*} => {
                        0 #(#fields_size)*
                    },
                });
                enum_sizes.push(quote! {
                    rapira::static_size([#(#fields_static_sizes)*]),
                });
                min_sizes.push(quote! {
                    rapira::min_size(&[#(#fields_min_sizes)*]),
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

    let static_size = match static_size {
        Some(static_size) => {
            quote! {
                #static_size
            }
        }
        None => {
            quote! {
                rapira::enum_size([#(#enum_sizes)*])
            }
        }
    };
    let min_size = match min_size {
        Some(min_size) => {
            quote! {
                #min_size
            }
        }
        None => {
            quote! {
                rapira::enum_min_size(&[#(#min_sizes)*])
            }
        }
    };

    let name_with_generics = build_ident(name, generics);

    let gen = quote! {
        #name_with_generics {
            const STATIC_SIZE: Option<usize> = #static_size;
            const MIN_SIZE: usize = #min_size;

            #[inline]
            fn from_slice(slice: &mut &[u8]) -> rapira::Result<Self>
            where
                Self: Sized,
            {
                let val: u8 = rapira::byte_rapira::from_slice(slice)?;
                match val {
                    #(#from_slice)*
                    _ => Err(rapira::RapiraError::EnumVariantError),
                }
            }

            #[inline]
            fn check_bytes(slice: &mut &[u8]) -> rapira::Result<()>
            where
                Self: Sized,
            {
                let val: u8 = rapira::byte_rapira::from_slice(slice)?;
                match val {
                    #(#check_bytes)*
                    _ => return Err(rapira::RapiraError::EnumVariantError),
                }
                Ok(())
            }

            #[inline]
            fn from_slice_unchecked(slice: &mut &[u8]) -> rapira::Result<Self>
            where
                Self: Sized,
            {
                let val: u8 = rapira::byte_rapira::from_slice(slice)?;
                match val {
                    #(#from_slice_unchecked)*
                    _ => Err(rapira::RapiraError::EnumVariantError),
                }
            }

            #[inline]
            unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> rapira::Result<Self>
            where
                Self: Sized,
            {
                let val: u8 = rapira::byte_rapira::from_slice_unsafe(slice)?;
                match val {
                    #(#from_slice_unsafe)*
                    _ => Err(rapira::RapiraError::EnumVariantError),
                }
            }

            #[inline]
            fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> rapira::Result<()> {
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
