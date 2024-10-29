extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::Ident;
use quote::quote;

pub fn simple_enum_serializer(name: &Ident) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(quote! {
        impl rapira::Rapira for #name {
            const STATIC_SIZE: Option<usize> = Some(1);
            const MIN_SIZE: usize = 1;

            #[inline]
            fn from_slice(slice: &mut &[u8]) -> rapira::Result<Self>
            where
                Self: Sized,
            {
                let val: u8 = rapira::byte_rapira::from_slice(slice)?;
                <Self as core::convert::TryFrom<u8>>::try_from(val).map_err(|_| rapira::RapiraError::EnumVariant)
            }

            #[inline]
            fn check_bytes(slice: &mut &[u8]) -> rapira::Result<()>
            where
                Self: Sized,
            {
                let val: u8 = rapira::byte_rapira::from_slice(slice)?;
                <Self as core::convert::TryFrom<u8>>::try_from(val).map_err(|_| rapira::RapiraError::EnumVariant)?;
                Ok(())
            }

            #[inline]
            unsafe fn from_slice_unchecked(slice: &mut &[u8]) -> rapira::Result<Self>
            where
                Self: Sized,
            {
                let val: u8 = rapira::byte_rapira::from_slice(slice)?;
                <Self as core::convert::TryFrom<u8>>::try_from(val).map_err(|_| rapira::RapiraError::EnumVariant)
            }

            #[inline]
            unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> rapira::Result<Self>
            where
                Self: Sized,
            {
                let val: u8 = rapira::byte_rapira::from_slice_unsafe(slice)?;
                <Self as core::convert::TryFrom<u8>>::try_from(val).map_err(|_| rapira::RapiraError::EnumVariant)
            }

            #[inline]
            fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> rapira::Result<()> {
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
    })
}
