extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{DataEnum, Field, Fields};

use crate::field_attrs::extract_idx_attr;

pub fn enum_with_primitive_serializer(
    data_enum: &DataEnum,
    name: &Ident,
    primitive_name: Ident,
) -> proc_macro::TokenStream {
    let variants_len = data_enum.variants.len();

    let mut from_slice: Vec<TokenStream> = Vec::with_capacity(variants_len);
    let mut check_bytes: Vec<TokenStream> = Vec::with_capacity(variants_len);
    let mut from_slice_unchecked: Vec<TokenStream> = Vec::with_capacity(variants_len);
    let mut from_slice_unsafe: Vec<TokenStream> = Vec::with_capacity(variants_len);
    let mut try_convert_to_bytes: Vec<TokenStream> = Vec::with_capacity(variants_len);
    let mut convert_to_bytes: Vec<TokenStream> = Vec::with_capacity(variants_len);
    let mut size: Vec<TokenStream> = Vec::with_capacity(variants_len);
    let mut enum_sizes: Vec<TokenStream> = Vec::with_capacity(variants_len);
    let mut min_sizes: Vec<TokenStream> = Vec::with_capacity(variants_len);

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

                min_sizes.push(quote! {
                    0usize,
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

                    min_sizes.push(quote! {
                        <#typ>::MIN_SIZE,
                    });
                } else {
                    let unnamed = &fields.unnamed;

                    let mut field_names: Vec<TokenStream> = Vec::with_capacity(len);
                    let mut unnamed_from_slice: Vec<TokenStream> = Vec::with_capacity(len);
                    let mut unnamed_check_bytes: Vec<TokenStream> = Vec::with_capacity(len);
                    let mut unnamed_from_slice_unchecked: Vec<TokenStream> =
                        Vec::with_capacity(len);
                    let mut unnamed_from_slice_unsafe: Vec<TokenStream> = Vec::with_capacity(len);
                    let mut unnamed_try_convert_to_bytes: Vec<TokenStream> =
                        Vec::with_capacity(len);
                    let mut unnamed_convert_to_bytes: Vec<TokenStream> = Vec::with_capacity(len);
                    let mut unnamed_size: Vec<TokenStream> = Vec::with_capacity(len);
                    let mut unnamed_static_sizes: Vec<TokenStream> = Vec::with_capacity(len);
                    let mut unnamed_min_sizes: Vec<TokenStream> = Vec::with_capacity(len);

                    for (idx, field) in unnamed.iter().enumerate() {
                        let typ = &field.ty;
                        let field_name = syn::Ident::new(&format!("arg{idx}"), Span::call_site());

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
                        unnamed_size.push(quote! { + (match <#typ>::STATIC_SIZE {
                            Some(s) => s,
                            None => #field_name.size()
                        }) });
                        unnamed_static_sizes.push(quote! {
                            <#typ>::STATIC_SIZE,
                        });
                        unnamed_min_sizes.push(quote! {
                            <#typ>::MIN_SIZE,
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

                    min_sizes.push(quote! {
                        rapira::min_size(&[#(#unnamed_min_sizes)*]),
                    });
                }
            }
            Fields::Named(fields) => {
                let named = &fields.named;
                let len = named.len();

                let mut fields_insert: Vec<(Field, u32)> = Vec::with_capacity(len);
                let mut seq = 0u32;

                for field in named.iter() {
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
                let mut named_from_slice: Vec<TokenStream> = Vec::with_capacity(len);
                let mut named_check_bytes: Vec<TokenStream> = Vec::with_capacity(len);
                let mut named_from_slice_unchecked: Vec<TokenStream> = Vec::with_capacity(len);
                let mut named_from_slice_unsafe: Vec<TokenStream> = Vec::with_capacity(len);
                let mut named_try_convert_to_bytes: Vec<TokenStream> = Vec::with_capacity(len);
                let mut named_convert_to_bytes: Vec<TokenStream> = Vec::with_capacity(len);
                let mut named_size: Vec<TokenStream> = Vec::with_capacity(len);
                let mut named_static_sizes: Vec<TokenStream> = Vec::with_capacity(len);
                let mut named_min_sizes: Vec<TokenStream> = Vec::with_capacity(len);

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
                    named_min_sizes.push(quote! {
                        <#typ>::MIN_SIZE,
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

                min_sizes.push(quote! {
                    rapira::min_size(&[#(#named_min_sizes)*]),
                });
            }
        };
    }

    let gen = quote! {
        impl rapira::Rapira for #name {
            const STATIC_SIZE: Option<usize> = rapira::enum_size([#(#enum_sizes)*]);
            const MIN_SIZE: usize = rapira::enum_min_size(&[#(#min_sizes)*]);

            #[inline]
            fn from_slice(slice: &mut &[u8]) -> rapira::Result<Self>
            where
                Self: Sized,
            {
                let val: u8 = rapira::byte_rapira::from_slice(slice)?;
                let t = <#primitive_name as TryFrom<u8>>::try_from(val).map_err(|_| rapira::RapiraError::EnumVariant)?;
                match t {
                    #(#from_slice)*
                }
            }

            #[inline]
            fn check_bytes(slice: &mut &[u8]) -> rapira::Result<()>
            where
                Self: Sized,
            {
                let val: u8 = rapira::byte_rapira::from_slice(slice)?;
                let t = <#primitive_name as TryFrom<u8>>::try_from(val).map_err(|_| rapira::RapiraError::EnumVariant)?;
                match t {
                    #(#check_bytes)*
                }
                Ok(())
            }

            #[inline]
            fn from_slice_unchecked(slice: &mut &[u8]) -> rapira::Result<Self>
            where
                Self: Sized,
            {
                let val: u8 = rapira::byte_rapira::from_slice(slice)?;
                let t = <#primitive_name as TryFrom<u8>>::try_from(val)
                    .map_err(|_| rapira::RapiraError::EnumVariant)?;
                match t {
                    #(#from_slice_unchecked)*
                }
            }

            #[inline]
            unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> rapira::Result<Self>
            where
                Self: Sized,
            {
                let val: u8 = rapira::byte_rapira::from_slice_unsafe(slice)?;
                let t = <#primitive_name as rapira::FromU8>::from_u8(val);
                match t {
                    #(#from_slice_unsafe)*
                }
            }

            #[inline]
            fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> rapira::Result<()> {
                let t = #primitive_name::from(self) as u8;
                rapira::push(slice, cursor, t);
                match self {
                    #(#try_convert_to_bytes)*
                }
                Ok(())
            }

            #[inline]
            fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
                let t = #primitive_name::from(self) as u8;
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
