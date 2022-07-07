extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, Data, DeriveInput, Field, Fields, Ident, Lit, LitInt, Meta, NestedMeta,
};

// TODO: impl for enum E { A { a_field1, ... }, B { b_field1... } } and enum E { A(B, C) }

fn get_primitive_name(ast: &DeriveInput) -> TokenStream {
    ast.attrs
        .iter()
        .find_map(|attr| {
            attr.path.segments.first().and_then(|segment| {
                if segment.ident != "coming" {
                    return None;
                }
                match attr.parse_args::<Meta>() {
                    Ok(Meta::NameValue(name_value)) => {
                        if name_value.path.to_token_stream().to_string() != "primitive" {
                            return None;
                        }
                        if let Lit::Str(lit_str) = name_value.lit {
                            let s = lit_str.parse::<Ident>().unwrap();
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
        .expect("complex enums must include primitive type name!")
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
                            let ident = field.ident.clone().unwrap();

                            let gen = quote! {
                               self.#ident.try_convert_to_bytes(slice, cursor)?;
                            };

                            gen
                        })
                        .collect();

                    let convert_to_bytes: Vec<TokenStream> = fields_insert
                        .iter()
                        .map(|(field, _)| {
                            let ident = field.ident.clone().unwrap();

                            let gen = quote! {
                               self.#ident.convert_to_bytes(slice, cursor);
                            };

                            gen
                        })
                        .collect();

                    let from_slice: Vec<TokenStream> = fields_insert
                        .iter()
                        .map(|(field, _)| {
                            let ident = field.ident.clone().unwrap();
                            let typ = field.ty.clone();

                            let gen = quote! {
                                let #ident = <#typ as rapira::Rapira>::from_slice(slice)?;
                            };

                            gen
                        })
                        .collect();

                    let check_bytes: Vec<TokenStream> = fields_insert
                        .iter()
                        .map(|(field, _)| {
                            let typ = field.ty.clone();
                            quote! {
                                <#typ>::check_bytes(slice)?;
                            }
                        })
                        .collect();

                    let from_slice_unchecked: Vec<TokenStream> = fields_insert
                        .iter()
                        .map(|(field, _)| {
                            let ident = field.ident.clone().unwrap();
                            let typ = field.ty.clone();

                            let gen = quote! {
                                let #ident = <#typ>::from_slice_unchecked(slice)?;
                            };

                            gen
                        })
                        .collect();

                    let from_slice_unsafe: Vec<TokenStream> = fields_insert
                        .iter()
                        .map(|(field, _)| {
                            let ident = field.ident.clone().unwrap();
                            let typ = field.ty.clone();

                            let gen = quote! {
                                let #ident = <#typ>::from_slice_unsafe(slice)?;
                            };

                            gen
                        })
                        .collect();

                    let field_names: Vec<TokenStream> = fields_insert
                        .iter()
                        .map(|(field, _)| {
                            let ident = field.ident.clone().unwrap();
                            quote! { #ident, }
                        })
                        .collect();

                    let size: Vec<TokenStream> = fields_insert
                        .iter()
                        .map(|(field, _)| {
                            let ident = field.ident.clone().unwrap();
                            let typ = field.ty.clone();

                            quote! { + (match <#typ>::STATIC_SIZE {
                                Some(s) => s,
                                None => self.#ident.size()
                            }) }
                        })
                        .collect();

                    let static_sizes: Vec<TokenStream> = fields_insert
                        .iter()
                        .map(|(field, _)| {
                            let typ = field.ty.clone();
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
                        let typ = field.ty.clone();
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
                let primitive_name: TokenStream = get_primitive_name(&ast);

                let mut from_slice: Vec<TokenStream> = Vec::with_capacity(variants_len);
                let mut check_bytes: Vec<TokenStream> = Vec::with_capacity(variants_len);
                let mut from_slice_unchecked: Vec<TokenStream> = Vec::with_capacity(variants_len);
                let mut from_slice_unsafe: Vec<TokenStream> = Vec::with_capacity(variants_len);
                let mut try_convert_to_bytes: Vec<TokenStream> = Vec::with_capacity(variants_len);
                let mut convert_to_bytes: Vec<TokenStream> = Vec::with_capacity(variants_len);
                let mut size: Vec<TokenStream> = Vec::with_capacity(variants_len);
                let mut enum_sizes: Vec<TokenStream> = Vec::with_capacity(variants_len);

                for variant in &data_enum.variants {
                    if variant.discriminant.is_some() {
                        // why? because discriminant number may not be equal to primitive number
                        panic!("enums variants with discriminant not support in current moment");
                    }
                    let fields = &variant.fields;
                    let field = match fields {
                        Fields::Unit => None,
                        Fields::Unnamed(fields) => {
                            let len = fields.unnamed.len();
                            if len != 1 {
                                panic!("enums variants is currently support only with 1 unnamed fields");
                            }
                            let field = fields.unnamed.first().unwrap();
                            Some(field.clone())
                        }
                        Fields::Named(_) => {
                            panic!("enums named variants is currently not support");
                        }
                    };
                    let variant_name = &variant.ident;

                    match &field {
                        Some(field) => {
                            let typ = field.ty.clone();

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

                            let try_convert_to_bytes_q = quote! {
                                #name::#variant_name(v) => {
                                    v.try_convert_to_bytes(slice, cursor)?;
                                }
                            };
                            try_convert_to_bytes.push(try_convert_to_bytes_q);

                            let convert_to_bytes_q = quote! {
                                #name::#variant_name(v) => {
                                    v.convert_to_bytes(slice, cursor);
                                }
                            };
                            convert_to_bytes.push(convert_to_bytes_q);

                            let size_q = quote! {
                                #name::#variant_name(v) => {
                                    match <#typ>::STATIC_SIZE {
                                        Some(s) => s,
                                        None => v.size(),
                                    }
                                },
                            };
                            size.push(size_q);

                            let enum_sizes_q = quote! {
                                <#typ>::STATIC_SIZE,
                            };
                            enum_sizes.push(enum_sizes_q);
                        }
                        None => {
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

                            let try_convert_to_bytes_q = quote! {
                                #name::#variant_name => {}
                            };
                            try_convert_to_bytes.push(try_convert_to_bytes_q);

                            let convert_to_bytes_q = quote! {
                                #name::#variant_name => {}
                            };
                            convert_to_bytes.push(convert_to_bytes_q);

                            let size_q = quote! {
                                #name::#variant_name => 0,
                            };
                            size.push(size_q);

                            let enum_sizes_q = quote! {
                                None,
                            };
                            enum_sizes.push(enum_sizes_q);
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
                            let t = <#primitive_name as core::convert::TryFrom<u8>>::try_from(val).map_err(|_| rapira::RapiraError::EnumVariantError)?;
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
        }
        Data::Union(_) => {
            panic!("unions not supported, but Rust enums is implemented GetType trait (use Enums instead)")
        }
    }
}
