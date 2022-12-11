#![cfg_attr(not(feature = "std"), no_std)]

pub mod error;

pub use error::{RapiraError, Result};

#[cfg(feature = "std")]
use std::net::IpAddr;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, collections::BTreeMap, string::String, vec::Vec};
#[cfg(feature = "arrayvec")]
use arrayvec::{ArrayString, ArrayVec};

#[cfg(feature = "decimal")]
use rust_decimal::Decimal;
#[cfg(feature = "smallvec")]
use smallvec::SmallVec;

use core::{mem::size_of, num::NonZeroU32};
pub use rapira_derive::Rapira;
use simdutf8::basic::from_utf8;

#[cfg(feature = "map")]
use indexmap::IndexMap;

#[cfg(feature = "map")]
use core::hash::BuildHasherDefault;

#[cfg(feature = "json")]
use serde_json::{Map, Number, Value};

pub trait Rapira {
    const STATIC_SIZE: Option<usize> = None;

    fn check_bytes(slice: &mut &[u8]) -> Result<()>;

    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized;

    /// this mean not unsafe, but unchecked (utf-8 strings not check...)
    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        Self::from_slice(slice)
    }

    /// # Safety
    ///
    /// This is unsafe
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized;

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        self.convert_to_bytes(slice, cursor);
        Ok(())
    }

    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize);

    fn size(&self) -> usize;

    #[cfg(feature = "alloc")]
    fn serialize(&self) -> Vec<u8> {
        let value_size = match Self::STATIC_SIZE {
            Some(s) => s,
            None => self.size(),
        };
        let mut bytes: Vec<u8> = vec![0u8; value_size];
        let mut cursor = 0usize;
        self.convert_to_bytes(&mut bytes, &mut cursor);
        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let mut bytes = bytes;
        Self::from_slice(&mut bytes)
    }

    fn deser_unchecked(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let mut bytes = bytes;
        Self::from_slice_unchecked(&mut bytes)
    }

    /// # Safety
    ///
    /// This is unsafe
    unsafe fn deser_unsafe(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let mut bytes = bytes;
        Self::from_slice_unsafe(&mut bytes)
    }
}

impl Rapira for () {
    const STATIC_SIZE: Option<usize> = Some(0);

    #[inline]
    fn check_bytes(_: &mut &[u8]) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn from_slice(_: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(())
    }

    #[inline]
    unsafe fn from_slice_unsafe(_: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(())
    }

    #[inline]
    fn convert_to_bytes(&self, _: &mut [u8], _: &mut usize) {}

    #[inline]
    fn size(&self) -> usize {
        0
    }
}

impl Rapira for bool {
    const STATIC_SIZE: Option<usize> = Some(1);

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let byte = *slice.first().ok_or(RapiraError::SliceLenError)?;

        *slice = unsafe { slice.get_unchecked(1..) };
        Ok(byte != 0)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        *slice = slice.get(1..).ok_or(RapiraError::SliceLenError)?;

        Ok(())
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let byte = *slice.get_unchecked(0);
        *slice = slice.get_unchecked(1..);
        Ok(byte != 0)
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        // println!("slice: {slice:?}, cursor: {cursor}");
        let byte = unsafe { slice.get_unchecked_mut(*cursor) };
        *byte = u8::from(*self);
        *cursor += 1;
    }

    #[inline]
    fn size(&self) -> usize {
        1
    }
}

pub mod byte_rapira {
    use super::*;

    pub const fn static_size<T>() -> Option<usize> {
        Some(1)
    }

    #[inline]
    pub fn size(_: &u8) -> usize {
        1
    }

    #[inline]
    pub fn check_bytes<T>(slice: &mut &[u8]) -> Result<()> {
        *slice = slice.get(1..).ok_or(RapiraError::SliceLenError)?;
        Ok(())
    }

    #[inline]
    pub fn from_slice(slice: &mut &[u8]) -> Result<u8> {
        let byte = *slice.first().ok_or(RapiraError::SliceLenError)?;
        *slice = unsafe { slice.get_unchecked(1..) };
        Ok(byte)
    }

    #[inline]
    pub fn from_slice_unchecked(slice: &mut &[u8]) -> Result<u8> {
        from_slice(slice)
    }

    /// ...
    ///
    /// # Errors
    ///
    /// This function will return an error if ...
    ///
    /// # Safety
    ///
    /// this is unsafe
    #[inline]
    pub unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<u8> {
        let byte = *slice.get_unchecked(0);
        *slice = slice.get_unchecked(1..);
        Ok(byte)
    }

    #[inline]
    pub fn convert_to_bytes(item: &u8, slice: &mut [u8], cursor: &mut usize) {
        let byte = unsafe { slice.get_unchecked_mut(*cursor) };
        *byte = *item;
        *cursor += 1;
    }

    #[inline]
    pub fn try_convert_to_bytes(item: &u8, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        let byte = unsafe { slice.get_unchecked_mut(*cursor) };
        *byte = *item;
        *cursor += 1;
        Ok(())
    }
}

macro_rules! impl_for_integer {
    ($type: ident) => {
        impl Rapira for $type {
            const STATIC_SIZE: Option<usize> = Some(size_of::<$type>());

            #[inline]
            fn from_slice(slice: &mut &[u8]) -> Result<Self>
            where
                Self: Sized,
            {
                let bytes: &[u8; size_of::<$type>()] = slice
                    .get(..size_of::<$type>())
                    .ok_or(RapiraError::SliceLenError)?
                    .try_into()
                    .unwrap();
                let u = <$type>::from_le_bytes(*bytes);

                *slice = unsafe { slice.get_unchecked(size_of::<$type>()..) };
                Ok(u)
            }

            #[inline]
            fn check_bytes(slice: &mut &[u8]) -> Result<()>
            where
                Self: Sized,
            {
                *slice = slice
                    .get(size_of::<$type>()..)
                    .ok_or(RapiraError::SliceLenError)?;

                Ok(())
            }

            #[inline]
            unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
            where
                Self: Sized,
            {
                let bytes: &[u8] = slice.get_unchecked(..size_of::<$type>());

                let arr: &[u8; size_of::<$type>()] = core::mem::transmute_copy(&bytes);
                let u = <$type>::from_le_bytes(*arr);

                // let u: $type = core::mem::transmute_copy(bytes);

                *slice = slice.get_unchecked(size_of::<$type>()..);
                Ok(u)
            }

            #[inline]
            fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
                let bytes = self.to_le_bytes();
                let end = *cursor + size_of::<$type>();
                let s = unsafe { slice.get_unchecked_mut(*cursor..end) };
                s.copy_from_slice(&bytes);
                *cursor = end;
            }

            #[inline]
            fn size(&self) -> usize {
                size_of::<$type>()
            }
        }
    };
}

impl_for_integer!(i8);
impl_for_integer!(i16);
impl_for_integer!(i32);
impl_for_integer!(i64);
impl_for_integer!(i128);
impl_for_integer!(u16);
impl_for_integer!(u32);
impl_for_integer!(u64);
impl_for_integer!(u128);

impl Rapira for usize {
    const STATIC_SIZE: Option<usize> = Some(size_of::<u64>());

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        u64::from_slice(slice).map(|u| u as usize)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        u64::check_bytes(slice)
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        u64::from_slice_unsafe(slice).map(|u| u as usize)
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        (*self as u64).convert_to_bytes(slice, cursor);
    }

    #[inline]
    fn size(&self) -> usize {
        size_of::<u64>()
    }
}

impl Rapira for isize {
    const STATIC_SIZE: Option<usize> = Some(size_of::<i64>());

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        i64::from_slice(slice).map(|u| u as isize)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        i64::check_bytes(slice)
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        i64::from_slice_unsafe(slice).map(|u| u as isize)
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        (*self as i64).convert_to_bytes(slice, cursor);
    }

    #[inline]
    fn size(&self) -> usize {
        size_of::<i64>()
    }
}

/// # Safety
///
/// This is unsafe.
#[inline]
pub unsafe fn get_u32_unsafe(slice: &mut &[u8]) -> u32 {
    let bytes: &[u8] = slice.get_unchecked(..4);

    let arr: &[u8; 4] = core::mem::transmute_copy(&bytes);
    let u = u32::from_le_bytes(*arr);

    *slice = slice.get_unchecked(4..);
    u
}

impl Rapira for NonZeroU32 {
    const STATIC_SIZE: Option<usize> = Some(size_of::<Self>());

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let u = u32::from_slice(slice)?;
        let u = NonZeroU32::new(u).ok_or(RapiraError::NonZeroError)?;

        *slice = unsafe { slice.get_unchecked(size_of::<Self>()..) };
        Ok(u)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        *slice = slice
            .get(size_of::<Self>()..)
            .ok_or(RapiraError::SliceLenError)?;

        Ok(())
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let u = u32::from_slice_unsafe(slice)?;
        let u = NonZeroU32::new_unchecked(u);

        *slice = slice.get_unchecked(size_of::<Self>()..);
        Ok(u)
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        let bytes = self.get();
        bytes.convert_to_bytes(slice, cursor);
    }

    #[inline]
    fn size(&self) -> usize {
        size_of::<Self>()
    }
}

impl Rapira for f32 {
    const STATIC_SIZE: Option<usize> = Some(size_of::<Self>());

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let bytes: &[u8; size_of::<Self>()] = slice
            .get(..size_of::<Self>())
            .ok_or(RapiraError::SliceLenError)?
            .try_into()
            .unwrap();
        let u = f32::from_le_bytes(*bytes);

        *slice = unsafe { slice.get_unchecked(size_of::<Self>()..) };
        Ok(u)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        let bytes: &[u8; size_of::<Self>()] = slice
            .get(..size_of::<Self>())
            .ok_or(RapiraError::SliceLenError)?
            .try_into()
            .unwrap();
        let u = Self::from_le_bytes(*bytes);

        if u.is_nan() {
            return Err(RapiraError::FloatIsNaNError);
        }

        *slice = unsafe { slice.get_unchecked(size_of::<Self>()..) };

        Ok(())
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let bytes: &[u8] = slice.get_unchecked(..size_of::<Self>());

        let arr: &[u8; size_of::<Self>()] = core::mem::transmute_copy(&bytes);

        let u = f32::from_le_bytes(*arr);

        *slice = slice.get_unchecked(size_of::<Self>()..);
        Ok(u)
    }

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        if self.is_nan() {
            return Err(RapiraError::FloatIsNaNError);
        }
        self.convert_to_bytes(slice, cursor);
        Ok(())
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        let bytes = self.to_le_bytes();
        let end = *cursor + size_of::<Self>();
        let s = unsafe { slice.get_unchecked_mut(*cursor..end) };
        s.copy_from_slice(&bytes);
        // unsafe {
        //     *slice = slice.get_unchecked_mut(size_of::<Self>()..) as &'a mut [u8];
        // };
        *cursor = end;
    }

    #[inline]
    fn size(&self) -> usize {
        4
    }
}

impl Rapira for f64 {
    const STATIC_SIZE: Option<usize> = Some(size_of::<Self>());

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let bytes: &[u8; size_of::<Self>()] = slice
            .get(..size_of::<Self>())
            .ok_or(RapiraError::SliceLenError)?
            .try_into()
            .unwrap();
        let u = f64::from_le_bytes(*bytes);

        *slice = unsafe { slice.get_unchecked(size_of::<Self>()..) };
        Ok(u)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        let bytes: &[u8; size_of::<Self>()] = slice
            .get(..size_of::<Self>())
            .ok_or(RapiraError::SliceLenError)?
            .try_into()
            .unwrap();
        let u = Self::from_le_bytes(*bytes);

        if u.is_nan() {
            return Err(RapiraError::FloatIsNaNError);
        }

        *slice = unsafe { slice.get_unchecked(size_of::<Self>()..) };

        Ok(())
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let bytes: &[u8] = slice.get_unchecked(..size_of::<Self>());

        let arr: &[u8; size_of::<Self>()] = core::mem::transmute_copy(&bytes);

        let u = f64::from_le_bytes(*arr);

        *slice = slice.get_unchecked(size_of::<Self>()..);
        Ok(u)
    }

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        if self.is_nan() {
            return Err(RapiraError::FloatIsNaNError);
        }
        self.convert_to_bytes(slice, cursor);
        Ok(())
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        let bytes = self.to_le_bytes();
        let end = *cursor + size_of::<Self>();
        let s = unsafe { slice.get_unchecked_mut(*cursor..end) };
        s.copy_from_slice(&bytes);
        // unsafe {
        //     *slice = slice.get_unchecked_mut(size_of::<Self>()..) as &'a mut [u8];
        // };
        *cursor = end;
    }

    #[inline]
    fn size(&self) -> usize {
        8
    }
}

#[cfg(feature = "alloc")]
impl Rapira for String {
    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;
        let bytes = slice.get(..len).ok_or(RapiraError::SliceLenError)?;
        let s = if len > 10 {
            from_utf8(bytes).map_err(|_| RapiraError::StringTypeError)?
        } else {
            core::str::from_utf8(bytes).map_err(|_| RapiraError::StringTypeError)?
        };

        let s = s.to_string();

        *slice = unsafe { slice.get_unchecked(len..) };
        Ok(s)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;
        let bytes = slice.get(..len).ok_or(RapiraError::SliceLenError)?;

        if len > 10 {
            from_utf8(bytes).map_err(|_| RapiraError::StringTypeError)?;
        } else {
            core::str::from_utf8(bytes).map_err(|_| RapiraError::StringTypeError)?;
        };

        *slice = unsafe { slice.get_unchecked(len..) };
        Ok(())
    }

    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;
        let bytes = slice.get(..len).ok_or(RapiraError::SliceLenError)?;
        let s = unsafe {
            let s = core::str::from_utf8_unchecked(bytes);
            s.to_string()
        };

        *slice = unsafe { slice.get_unchecked(len..) };
        Ok(s)
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = get_u32_unsafe(slice) as usize;
        let bytes = slice.get_unchecked(..len);

        let s = core::str::from_utf8_unchecked(bytes);
        let s = s.to_string();

        *slice = slice.get_unchecked(len..);
        Ok(s)
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        let len = self.len() as u32;
        len.convert_to_bytes(slice, cursor);
        extend(slice, cursor, self.as_bytes());
    }

    #[inline]
    fn size(&self) -> usize {
        4 + self.len()
    }
}

#[cfg(feature = "alloc")]
impl Rapira for Vec<u8> {
    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;
        let bytes = slice.get(..len).ok_or(RapiraError::SliceLenError)?;

        *slice = unsafe { slice.get_unchecked(len..) };
        Ok(bytes.to_vec())
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;

        *slice = slice.get(len..).ok_or(RapiraError::SliceLenError)?;
        Ok(())
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = get_u32_unsafe(slice) as usize;
        let bytes = slice.get_unchecked(..len);

        *slice = slice.get_unchecked(len..);
        Ok(bytes.to_vec())
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        let len = self.len() as u32;
        len.convert_to_bytes(slice, cursor);
        extend(slice, cursor, self);
    }

    #[inline]
    fn size(&self) -> usize {
        4 + self.len()
    }
}

impl<const CAP: usize> Rapira for [u8; CAP] {
    const STATIC_SIZE: Option<usize> = Some(CAP);

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let bytes: [u8; CAP] = slice
            .get(..CAP)
            .ok_or(RapiraError::SliceLenError)?
            .try_into()
            .unwrap();

        *slice = unsafe { slice.get_unchecked(CAP..) };
        Ok(bytes)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        *slice = slice.get(CAP..).ok_or(RapiraError::SliceLenError)?;
        Ok(())
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let bytes: &[u8] = slice.get_unchecked(..CAP);

        let arr: &[u8; CAP] = core::mem::transmute_copy(&bytes);

        *slice = slice.get_unchecked(CAP..);
        Ok(*arr)
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        let end = *cursor + CAP;
        let s = unsafe { slice.get_unchecked_mut(*cursor..end) };
        s.copy_from_slice(self);
        *cursor = end;
    }

    #[inline]
    fn size(&self) -> usize {
        CAP
    }
}

impl<T: Rapira, const CAP: usize> Rapira for [T; CAP] {
    const STATIC_SIZE: Option<usize> = static_size([T::STATIC_SIZE; CAP]);

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()> {
        for _ in 0..CAP {
            T::check_bytes(slice)?;
        }

        Ok(())
    }

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let mut vec: Vec<T> = Vec::with_capacity(CAP);

        for _ in 0..CAP {
            let val = T::from_slice(slice)?;
            vec.push(val);
        }

        let arr: [T; CAP] = vec.try_into().map_err(|_| RapiraError::SliceLenError)?;

        Ok(arr)
    }

    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let mut vec: Vec<T> = Vec::with_capacity(CAP);

        for _ in 0..CAP {
            let val = T::from_slice_unchecked(slice)?;
            vec.push(val);
        }

        let arr: [T; CAP] = vec.try_into().map_err(|_| RapiraError::SliceLenError)?;

        Ok(arr)
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let mut vec: Vec<T> = Vec::with_capacity(CAP);

        for _ in 0..CAP {
            let val = T::from_slice_unsafe(slice)?;
            vec.push(val);
        }

        let arr: [T; CAP] = vec.try_into().map_err(|_| RapiraError::SliceLenError)?;

        Ok(arr)
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        for val in self.iter() {
            val.convert_to_bytes(slice, cursor);
        }
    }

    #[inline]
    fn size(&self) -> usize {
        match T::STATIC_SIZE {
            Some(size) => size * CAP,
            None => self.iter().map(|i| i.size()).sum(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<T: Rapira> Rapira for Vec<T> {
    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;
        let mut vec: Vec<T> = Vec::with_capacity(len);

        for _ in 0..len {
            let val = T::from_slice(slice)?;
            vec.push(val);
        }

        Ok(vec)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;

        for _ in 0..len {
            T::check_bytes(slice)?;
        }

        Ok(())
    }

    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;
        let mut vec: Vec<T> = Vec::with_capacity(len);

        for _ in 0..len {
            let val = T::from_slice_unchecked(slice)?;
            vec.push(val);
        }

        Ok(vec)
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = get_u32_unsafe(slice) as usize;
        let mut vec: Vec<T> = Vec::with_capacity(len);

        for _ in 0..len {
            let val = T::from_slice_unsafe(slice)?;
            vec.push(val);
        }

        Ok(vec)
    }

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        let len = self.len() as u32;
        len.try_convert_to_bytes(slice, cursor)?;

        for val in self.iter() {
            val.try_convert_to_bytes(slice, cursor)?;
        }

        Ok(())
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        let len = self.len() as u32;
        len.convert_to_bytes(slice, cursor);

        for val in self.iter() {
            val.convert_to_bytes(slice, cursor);
        }
    }

    #[inline]
    fn size(&self) -> usize {
        match T::STATIC_SIZE {
            Some(size) => 4 + (size * self.len()),
            None => 4 + self.iter().fold(0, |b, v| b + v.size()),
        }
    }
}

#[cfg(feature = "arrayvec")]
impl<T: Rapira, const CAP: usize> Rapira for ArrayVec<T, CAP> {
    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;
        let mut vec: ArrayVec<T, CAP> = ArrayVec::new_const();

        for _ in 0..len {
            let val = T::from_slice(slice)?;
            vec.push(val);
        }

        Ok(vec)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;

        for _ in 0..len {
            T::check_bytes(slice)?;
        }

        Ok(())
    }

    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;
        let mut vec: ArrayVec<T, CAP> = ArrayVec::new_const();

        for _ in 0..len {
            let val = T::from_slice_unchecked(slice)?;
            vec.push(val);
        }

        Ok(vec)
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = get_u32_unsafe(slice) as usize;
        let mut vec: ArrayVec<T, CAP> = ArrayVec::new_const();

        for _ in 0..len {
            let val = T::from_slice_unsafe(slice)?;
            vec.push(val);
        }

        Ok(vec)
    }

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        let len = self.len() as u32;
        len.try_convert_to_bytes(slice, cursor)?;

        for val in self.iter() {
            val.try_convert_to_bytes(slice, cursor)?;
        }

        Ok(())
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        let len = self.len() as u32;
        len.convert_to_bytes(slice, cursor);

        for val in self.iter() {
            val.convert_to_bytes(slice, cursor);
        }
    }

    #[inline]
    fn size(&self) -> usize {
        match T::STATIC_SIZE {
            Some(size) => 4 + (size * self.len()),
            None => 4 + self.iter().fold(0, |b, v| b + v.size()),
        }
    }
}

#[cfg(feature = "arrayvec")]
impl<const CAP: usize> Rapira for ArrayString<CAP> {
    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;
        let bytes = slice.get(..len).ok_or(RapiraError::SliceLenError)?;
        let s = if len > 10 {
            from_utf8(bytes).map_err(|_| RapiraError::StringTypeError)?
        } else {
            core::str::from_utf8(bytes).map_err(|_| RapiraError::StringTypeError)?
        };

        let s = ArrayString::from(s).unwrap();

        *slice = unsafe { slice.get_unchecked(len..) };
        Ok(s)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;
        let bytes = slice.get(..len).ok_or(RapiraError::SliceLenError)?;

        if len > 10 {
            from_utf8(bytes).map_err(|_| RapiraError::StringTypeError)?;
        } else {
            core::str::from_utf8(bytes).map_err(|_| RapiraError::StringTypeError)?;
        };

        *slice = unsafe { slice.get_unchecked(len..) };
        Ok(())
    }

    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;
        let bytes = slice.get(..len).ok_or(RapiraError::SliceLenError)?;
        let s = unsafe {
            let s = core::str::from_utf8_unchecked(bytes);
            ArrayString::from(s).unwrap()
        };

        *slice = unsafe { slice.get_unchecked(len..) };
        Ok(s)
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = get_u32_unsafe(slice) as usize;
        let bytes = slice.get_unchecked(..len);
        let s = core::str::from_utf8_unchecked(bytes);
        let s = ArrayString::from(s).unwrap();

        *slice = slice.get_unchecked(len..);
        Ok(s)
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        let len = self.len() as u32;
        len.convert_to_bytes(slice, cursor);

        extend(slice, cursor, self.as_bytes());
    }

    #[inline]
    fn size(&self) -> usize {
        4 + self.len()
    }
}

#[cfg(feature = "smallvec")]
impl<T: Rapira, const CAP: usize> Rapira for SmallVec<[T; CAP]> {
    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;
        let mut vec: SmallVec<[T; CAP]> = SmallVec::new_const();

        for _ in 0..len {
            let val = T::from_slice(slice)?;
            vec.push(val);
        }

        Ok(vec)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;

        for _ in 0..len {
            T::check_bytes(slice)?;
        }

        Ok(())
    }

    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;
        let mut vec: SmallVec<[T; CAP]> = SmallVec::new_const();

        for _ in 0..len {
            let val = T::from_slice_unchecked(slice)?;
            vec.push(val);
        }

        Ok(vec)
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = get_u32_unsafe(slice) as usize;
        let mut vec: SmallVec<[T; CAP]> = SmallVec::new_const();

        for _ in 0..len {
            let val = T::from_slice_unsafe(slice)?;
            vec.push(val);
        }

        Ok(vec)
    }

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        let len = self.len() as u32;
        len.try_convert_to_bytes(slice, cursor)?;

        for val in self.iter() {
            val.try_convert_to_bytes(slice, cursor)?;
        }

        Ok(())
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        let len = self.len() as u32;
        len.convert_to_bytes(slice, cursor);

        for val in self.iter() {
            val.convert_to_bytes(slice, cursor);
        }
    }

    #[inline]
    fn size(&self) -> usize {
        match T::STATIC_SIZE {
            Some(size) => 4 + (size * self.len()),
            None => 4 + self.iter().fold(0, |b, v| b + v.size()),
        }
    }
}

pub const fn enum_static_size(s: Option<usize>) -> Option<usize> {
    match s {
        Some(s) => Some(s + 1),
        None => None,
    }
}

#[inline]
pub fn push(slice: &mut [u8], cursor: &mut usize, item: u8) {
    let s = unsafe { slice.get_unchecked_mut(*cursor) };
    *s = item;
    *cursor += 1;
    // unsafe {
    //     *slice = slice.get_unchecked_mut(1..);
    // };
}

#[inline]
pub fn extend(slice: &mut [u8], cursor: &mut usize, items: &[u8]) {
    let end = *cursor + items.len();
    let s = unsafe { slice.get_unchecked_mut(*cursor..end) };
    s.copy_from_slice(items);
    *cursor = end;
    // unsafe {
    //     *slice = slice.get_unchecked_mut(items.len()..);
    // };
}

impl<T: Rapira> Rapira for Option<T> {
    const STATIC_SIZE: Option<usize> = enum_static_size(T::STATIC_SIZE);

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let b = byte_rapira::from_slice(slice)?;
        if b != 0 {
            let t = T::from_slice(slice)?;
            Ok(Some(t))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        let b = byte_rapira::from_slice(slice)?;
        if b != 0 {
            T::check_bytes(slice)?;
        }

        Ok(())
    }

    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let b = byte_rapira::from_slice(slice)?;
        if b != 0 {
            let t = T::from_slice_unchecked(slice)?;
            Ok(Some(t))
        } else {
            Ok(None)
        }
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let b = byte_rapira::from_slice_unsafe(slice)?;
        if b != 0 {
            let t = T::from_slice_unsafe(slice)?;
            Ok(Some(t))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        match self.as_ref() {
            Some(s) => {
                push(slice, cursor, 1);
                s.try_convert_to_bytes(slice, cursor)?;
            }
            None => {
                push(slice, cursor, 0);
            }
        }
        Ok(())
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        match self.as_ref() {
            Some(s) => {
                push(slice, cursor, 1);
                s.convert_to_bytes(slice, cursor);
            }
            None => {
                push(slice, cursor, 0);
            }
        }
    }

    #[inline]
    fn size(&self) -> usize {
        match self {
            None => 1,
            Some(t) => 1 + t.size(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<T: Rapira> Rapira for Box<T> {
    const STATIC_SIZE: Option<usize> = T::STATIC_SIZE;

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let t = T::from_slice(slice)?;
        Ok(Box::new(t))
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        T::check_bytes(slice)
    }

    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let t = T::from_slice_unchecked(slice)?;
        Ok(Box::new(t))
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let t = T::from_slice_unsafe(slice)?;
        Ok(Box::new(t))
    }

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        self.as_ref().try_convert_to_bytes(slice, cursor)
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        self.as_ref().convert_to_bytes(slice, cursor)
    }

    #[inline]
    fn size(&self) -> usize {
        self.as_ref().size()
    }
}

pub const fn static_size<const N: usize>(arr: [Option<usize>; N]) -> Option<usize> {
    let mut i = 0;
    let mut size = 0;
    let mut is_static = true;
    while i < arr.len() {
        let item = arr[i];
        match item {
            Some(s) => {
                size += s;
            }
            None => {
                is_static = false;
                break;
            }
        }
        i += 1;
    }
    if is_static {
        Some(size)
    } else {
        None
    }
}

impl<T0: Rapira, T1: Rapira> Rapira for (T0, T1) {
    const STATIC_SIZE: Option<usize> = static_size([T0::STATIC_SIZE, T1::STATIC_SIZE]);

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let t0 = T0::from_slice(slice)?;
        let t1 = T1::from_slice(slice)?;
        Ok((t0, t1))
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        T0::check_bytes(slice)?;
        T1::check_bytes(slice)?;
        Ok(())
    }

    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let t0 = T0::from_slice_unchecked(slice)?;
        let t1 = T1::from_slice_unchecked(slice)?;
        Ok((t0, t1))
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let t0 = T0::from_slice_unsafe(slice)?;
        let t1 = T1::from_slice_unsafe(slice)?;
        Ok((t0, t1))
    }

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        self.0.try_convert_to_bytes(slice, cursor)?;
        self.1.try_convert_to_bytes(slice, cursor)?;
        Ok(())
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        self.0.convert_to_bytes(slice, cursor);
        self.1.convert_to_bytes(slice, cursor);
    }

    #[inline]
    fn size(&self) -> usize {
        let a = match T0::STATIC_SIZE {
            Some(s) => s,
            None => self.0.size(),
        };
        let b = match T1::STATIC_SIZE {
            Some(s) => s,
            None => self.1.size(),
        };
        a + b
    }
}

impl<T0: Rapira, T1: Rapira, T2: Rapira> Rapira for (T0, T1, T2) {
    const STATIC_SIZE: Option<usize> =
        static_size([T0::STATIC_SIZE, T1::STATIC_SIZE, T2::STATIC_SIZE]);

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let t0 = T0::from_slice(slice)?;
        let t1 = T1::from_slice(slice)?;
        let t2 = T2::from_slice(slice)?;
        Ok((t0, t1, t2))
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        T0::check_bytes(slice)?;
        T1::check_bytes(slice)?;
        T2::check_bytes(slice)?;
        Ok(())
    }

    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let t0 = T0::from_slice_unchecked(slice)?;
        let t1 = T1::from_slice_unchecked(slice)?;
        let t2 = T2::from_slice_unchecked(slice)?;
        Ok((t0, t1, t2))
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let t0 = T0::from_slice_unsafe(slice)?;
        let t1 = T1::from_slice_unsafe(slice)?;
        let t2 = T2::from_slice_unsafe(slice)?;
        Ok((t0, t1, t2))
    }

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        self.0.try_convert_to_bytes(slice, cursor)?;
        self.1.try_convert_to_bytes(slice, cursor)?;
        self.2.try_convert_to_bytes(slice, cursor)?;
        Ok(())
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        self.0.convert_to_bytes(slice, cursor);
        self.1.convert_to_bytes(slice, cursor);
        self.2.convert_to_bytes(slice, cursor);
    }

    #[inline]
    fn size(&self) -> usize {
        (match T0::STATIC_SIZE {
            Some(s) => s,
            None => self.0.size(),
        }) + (match T1::STATIC_SIZE {
            Some(s) => s,
            None => self.1.size(),
        }) + (match T2::STATIC_SIZE {
            Some(s) => s,
            None => self.2.size(),
        })
    }
}

impl<T0: Rapira, T1: Rapira, T2: Rapira, T3: Rapira> Rapira for (T0, T1, T2, T3) {
    const STATIC_SIZE: Option<usize> = static_size([
        T0::STATIC_SIZE,
        T1::STATIC_SIZE,
        T2::STATIC_SIZE,
        T3::STATIC_SIZE,
    ]);

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let t0 = T0::from_slice(slice)?;
        let t1 = T1::from_slice(slice)?;
        let t2 = T2::from_slice(slice)?;
        let t3 = T3::from_slice(slice)?;
        Ok((t0, t1, t2, t3))
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        T0::check_bytes(slice)?;
        T1::check_bytes(slice)?;
        T2::check_bytes(slice)?;
        T3::check_bytes(slice)?;
        Ok(())
    }

    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let t0 = T0::from_slice_unchecked(slice)?;
        let t1 = T1::from_slice_unchecked(slice)?;
        let t2 = T2::from_slice_unchecked(slice)?;
        let t3 = T3::from_slice_unchecked(slice)?;
        Ok((t0, t1, t2, t3))
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let t0 = T0::from_slice_unsafe(slice)?;
        let t1 = T1::from_slice_unsafe(slice)?;
        let t2 = T2::from_slice_unsafe(slice)?;
        let t3 = T3::from_slice_unsafe(slice)?;
        Ok((t0, t1, t2, t3))
    }

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        self.0.try_convert_to_bytes(slice, cursor)?;
        self.1.try_convert_to_bytes(slice, cursor)?;
        self.2.try_convert_to_bytes(slice, cursor)?;
        self.3.try_convert_to_bytes(slice, cursor)?;
        Ok(())
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        self.0.convert_to_bytes(slice, cursor);
        self.1.convert_to_bytes(slice, cursor);
        self.2.convert_to_bytes(slice, cursor);
        self.3.convert_to_bytes(slice, cursor);
    }

    #[inline]
    fn size(&self) -> usize {
        (match T0::STATIC_SIZE {
            Some(s) => s,
            None => self.0.size(),
        }) + (match T1::STATIC_SIZE {
            Some(s) => s,
            None => self.1.size(),
        }) + (match T2::STATIC_SIZE {
            Some(s) => s,
            None => self.2.size(),
        }) + (match T3::STATIC_SIZE {
            Some(s) => s,
            None => self.3.size(),
        })
    }
}

#[cfg(feature = "alloc")]
impl<K: Rapira, V: Rapira> Rapira for BTreeMap<K, V>
where
    K: Ord,
{
    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self> {
        let len = u32::from_slice(slice)? as usize;
        let mut map = BTreeMap::<K, V>::new();
        for _ in 0..len {
            let key = K::from_slice(slice)?;
            let value = V::from_slice(slice)?;
            map.insert(key, value);
        }
        Ok(map)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;

        for _ in 0..len {
            K::check_bytes(slice)?;
            V::check_bytes(slice)?;
        }

        Ok(())
    }

    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self> {
        let len = u32::from_slice(slice)? as usize;
        let mut map = BTreeMap::<K, V>::new();
        for _ in 0..len {
            let key = K::from_slice_unchecked(slice)?;
            let value = V::from_slice_unchecked(slice)?;
            map.insert(key, value);
        }
        Ok(map)
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = get_u32_unsafe(slice) as usize;
        let mut map = BTreeMap::<K, V>::new();
        for _ in 0..len {
            let key = K::from_slice_unsafe(slice)?;
            let value = V::from_slice_unsafe(slice)?;
            map.insert(key, value);
        }
        Ok(map)
    }

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        let len = self.len() as u32;
        len.try_convert_to_bytes(slice, cursor)?;
        for (key, value) in self {
            key.try_convert_to_bytes(slice, cursor)?;
            value.try_convert_to_bytes(slice, cursor)?;
        }
        Ok(())
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        let len = self.len() as u32;
        len.convert_to_bytes(slice, cursor);
        for (key, value) in self {
            key.convert_to_bytes(slice, cursor);
            value.convert_to_bytes(slice, cursor);
        }
    }

    #[inline]
    fn size(&self) -> usize {
        if let Some(k) = K::STATIC_SIZE {
            if let Some(v) = V::STATIC_SIZE {
                4 + (self.len() * (k + v))
            } else {
                4 + (k * self.len()) + self.iter().fold(0, |b, (_, v)| b + v.size())
            }
        } else {
            4 + self.iter().fold(0, |b, (k, v)| {
                b + k.size() + V::STATIC_SIZE.unwrap_or_else(|| v.size())
            })
        }
    }
}

#[cfg(feature = "map")]
impl<K: Rapira, V: Rapira, S> Rapira for IndexMap<K, V, BuildHasherDefault<S>>
where
    K: Eq + core::hash::Hash,
    S: core::hash::Hasher + core::default::Default,
{
    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;
        let hasher = BuildHasherDefault::<S>::default();
        let mut map =
            IndexMap::<K, V, BuildHasherDefault<S>>::with_capacity_and_hasher(len, hasher);
        for _ in 0..len {
            let key = K::from_slice(slice)?;
            let value = V::from_slice(slice)?;
            map.insert(key, value);
        }
        Ok(map)
    }
    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;
        for _ in 0..len {
            K::check_bytes(slice)?;
            V::check_bytes(slice)?;
        }
        Ok(())
    }
    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;
        let hasher = BuildHasherDefault::<S>::default();
        let mut map =
            IndexMap::<K, V, BuildHasherDefault<S>>::with_capacity_and_hasher(len, hasher);
        for _ in 0..len {
            let key = K::from_slice_unchecked(slice)?;
            let value = V::from_slice_unchecked(slice)?;
            map.insert(key, value);
        }
        Ok(map)
    }
    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = get_u32_unsafe(slice) as usize;
        let hasher = BuildHasherDefault::<S>::default();
        let mut map =
            IndexMap::<K, V, BuildHasherDefault<S>>::with_capacity_and_hasher(len, hasher);
        for _ in 0..len {
            let key = K::from_slice_unsafe(slice)?;
            let value = V::from_slice_unsafe(slice)?;
            map.insert(key, value);
        }
        Ok(map)
    }
    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        let len = self.len() as u32;
        len.try_convert_to_bytes(slice, cursor)?;
        for (key, value) in self {
            key.try_convert_to_bytes(slice, cursor)?;
            value.try_convert_to_bytes(slice, cursor)?;
        }
        Ok(())
    }
    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        let len = self.len() as u32;
        len.convert_to_bytes(slice, cursor);
        for (key, value) in self {
            key.convert_to_bytes(slice, cursor);
            value.convert_to_bytes(slice, cursor);
        }
    }
    #[inline]
    fn size(&self) -> usize {
        if let Some(k) = K::STATIC_SIZE {
            if let Some(v) = V::STATIC_SIZE {
                4 + (self.len() * (k + v))
            } else {
                4 + (k * self.len()) + self.iter().fold(0, |b, (_, v)| b + v.size())
            }
        } else {
            4 + self.iter().fold(0, |b, (k, v)| {
                b + k.size() + V::STATIC_SIZE.unwrap_or_else(|| v.size())
            })
        }
    }
}

#[cfg(feature = "std")]
impl Rapira for IpAddr {
    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let b = byte_rapira::from_slice(slice)?;
        if b == 0 {
            let v4 = <[u8; 4]>::from_slice(slice)?;
            Ok(IpAddr::from(v4))
        } else {
            let v6 = <[u8; 16]>::from_slice(slice)?;
            Ok(IpAddr::from(v6))
        }
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        let b = byte_rapira::from_slice(slice)?;

        if b == 0 {
            <[u8; 4]>::check_bytes(slice)?;
        } else {
            <[u8; 16]>::check_bytes(slice)?;
        }

        Ok(())
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let b = byte_rapira::from_slice_unsafe(slice)?;
        if b == 0 {
            let v4 = <[u8; 4]>::from_slice_unsafe(slice)?;
            Ok(IpAddr::from(v4))
        } else {
            let v6 = <[u8; 16]>::from_slice_unsafe(slice)?;
            Ok(IpAddr::from(v6))
        }
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        match self {
            IpAddr::V4(v4) => {
                push(slice, cursor, 0);
                v4.octets().convert_to_bytes(slice, cursor);
            }
            IpAddr::V6(v6) => {
                push(slice, cursor, 1);
                v6.octets().convert_to_bytes(slice, cursor);
            }
        }
    }

    #[inline]
    fn size(&self) -> usize {
        1 + match self {
            IpAddr::V4(_) => 4,
            IpAddr::V6(_) => 16,
        }
    }
}

pub const fn enum_size<const N: usize>(arr: [Option<usize>; N]) -> Option<usize> {
    let mut i = 0;
    let mut size = 0;
    let mut is_static = true;
    let mut is_init = false;
    while i < arr.len() {
        let item = arr[i];
        match item {
            Some(s) => {
                if !is_init {
                    size = s;
                    is_init = true;
                } else if s != size {
                    is_static = false;
                    break;
                }
            }
            None => {
                is_static = false;
                break;
            }
        }
        i += 1;
    }
    if is_static {
        Some(size + 1)
    } else {
        None
    }
}

#[cfg(feature = "zerocopy")]
pub mod zero {
    use core::mem::size_of;

    use super::*;

    #[cfg(feature = "zerocopy")]
    use zerocopy::{AsBytes, FromBytes};

    pub const fn static_size<T>() -> Option<usize>
    where
        T: FromBytes + AsBytes + Sized,
    {
        Some(size_of::<T>())
    }

    #[inline]
    pub fn check_bytes<T>(slice: &mut &[u8]) -> Result<()>
    where
        T: FromBytes + AsBytes + Sized,
    {
        let size = size_of::<T>();

        *slice = slice.get(size..).ok_or(RapiraError::SliceLenError)?;

        Ok(())
    }

    #[inline]
    pub fn from_slice<T>(slice: &mut &[u8]) -> Result<T>
    where
        T: FromBytes + AsBytes + Sized,
    {
        let size = size_of::<T>();
        let bytes: &[u8] = slice.get(..size).ok_or(RapiraError::SliceLenError)?;

        *slice = unsafe { slice.get_unchecked(size..) };

        let t: T = FromBytes::read_from(bytes).unwrap();
        Ok(t)
    }

    #[inline]
    pub fn from_slice_unchecked<T>(slice: &mut &[u8]) -> Result<T>
    where
        T: FromBytes + AsBytes + Sized,
    {
        from_slice(slice)
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    ///
    /// # Safety
    ///
    /// .
    #[inline]
    pub unsafe fn from_slice_unsafe<T>(slice: &mut &[u8]) -> Result<T>
    where
        T: FromBytes + AsBytes + Sized,
    {
        let size = size_of::<T>();
        let bytes: &[u8] = slice.get_unchecked(..size);

        *slice = slice.get_unchecked(size..);

        let t: T = FromBytes::read_from(bytes).unwrap();
        Ok(t)
    }

    #[inline]
    pub fn convert_to_bytes<T>(item: &T, slice: &mut [u8], cursor: &mut usize)
    where
        T: FromBytes + AsBytes + Sized,
    {
        let bytes = item.as_bytes();
        extend(slice, cursor, bytes);
    }

    #[inline]
    pub fn try_convert_to_bytes<T>(item: &T, slice: &mut [u8], cursor: &mut usize) -> Result<()>
    where
        T: FromBytes + AsBytes + Sized,
    {
        convert_to_bytes(item, slice, cursor);
        Ok(())
    }

    #[inline]
    pub fn size<T>(_: &T) -> usize
    where
        T: FromBytes + AsBytes + Sized,
    {
        size_of::<T>()
    }
}

#[cfg(feature = "json")]
impl Rapira for Value {
    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let byte = byte_rapira::from_slice(slice)?;
        match byte {
            0 => Ok(Value::Null),
            1 => {
                let b = bool::from_slice(slice)?;
                Ok(Value::Bool(b))
            }
            2 => {
                let byte = byte_rapira::from_slice(slice)?;
                if byte == 0 {
                    let u = u64::from_slice(slice)?;
                    Ok(Value::Number(u.into()))
                } else if byte == 1 {
                    let i = i64::from_slice(slice)?;
                    Ok(Value::Number(i.into()))
                } else if byte == 2 {
                    let f = f64::from_slice(slice)?;
                    let number = Number::from_f64(f).ok_or(RapiraError::FloatIsNaNError)?;
                    Ok(Value::Number(number))
                } else {
                    Err(RapiraError::EnumVariantError)
                }
            }
            3 => {
                let s = String::from_slice(slice)?;
                Ok(Value::String(s))
            }
            4 => {
                let vec = Vec::<Value>::from_slice(slice)?;
                Ok(Value::Array(vec))
            }
            5 => {
                let len = u32::from_slice(slice)? as usize;
                let mut map = Map::new();
                for _ in 0..len {
                    let key = String::from_slice(slice)?;
                    let val = Value::from_slice(slice)?;
                    map.insert(key, val);
                }
                Ok(Value::Object(map))
            }
            _ => Err(RapiraError::EnumVariantError),
        }
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        let byte = byte_rapira::from_slice(slice)?;
        match byte {
            0 => {}
            1 => {
                bool::check_bytes(slice)?;
            }
            2 => {
                let byte = byte_rapira::from_slice(slice)?;
                if byte == 0 {
                    u64::check_bytes(slice)?;
                } else if byte == 1 {
                    i64::check_bytes(slice)?;
                } else if byte == 2 {
                    f64::check_bytes(slice)?;
                } else {
                    return Err(RapiraError::EnumVariantError);
                }
            }
            3 => {
                String::check_bytes(slice)?;
            }
            4 => {
                Vec::<Value>::check_bytes(slice)?;
            }
            5 => {
                let len = u32::from_slice(slice)? as usize;
                for _ in 0..len {
                    String::check_bytes(slice)?;
                    Value::check_bytes(slice)?;
                }
            }
            _ => return Err(RapiraError::EnumVariantError),
        }

        Ok(())
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let byte = byte_rapira::from_slice_unsafe(slice)?;
        match byte {
            0 => Ok(Value::Null),
            1 => {
                let b = bool::from_slice_unsafe(slice)?;
                Ok(Value::Bool(b))
            }
            2 => {
                let byte = byte_rapira::from_slice_unsafe(slice)?;
                if byte == 0 {
                    let u = u64::from_slice_unsafe(slice)?;
                    Ok(Value::Number(u.into()))
                } else if byte == 1 {
                    let i = i64::from_slice_unsafe(slice)?;
                    Ok(Value::Number(i.into()))
                } else if byte == 2 {
                    let f = f64::from_slice_unsafe(slice)?;
                    let number = Number::from_f64(f).ok_or(RapiraError::FloatIsNaNError)?;
                    Ok(Value::Number(number))
                } else {
                    Err(RapiraError::EnumVariantError)
                }
            }
            3 => {
                let s = String::from_slice_unsafe(slice)?;
                Ok(Value::String(s))
            }
            4 => {
                let vec = Vec::<Value>::from_slice_unsafe(slice)?;
                Ok(Value::Array(vec))
            }
            5 => {
                let len = get_u32_unsafe(slice) as usize;
                let mut map = Map::new();
                for _ in 0..len {
                    let key = String::from_slice_unsafe(slice)?;
                    let val = Value::from_slice_unsafe(slice)?;
                    map.insert(key, val);
                }
                Ok(Value::Object(map))
            }
            _ => Err(RapiraError::EnumVariantError),
        }
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        match self {
            Value::Null => {
                push(slice, cursor, 0);
            }
            Value::Bool(v) => {
                push(slice, cursor, 1);
                v.convert_to_bytes(slice, cursor);
            }
            Value::Number(n) => {
                push(slice, cursor, 2);
                if let Some(u) = n.as_u64() {
                    push(slice, cursor, 0);
                    u.convert_to_bytes(slice, cursor);
                } else if let Some(i) = n.as_i64() {
                    push(slice, cursor, 1);
                    i.convert_to_bytes(slice, cursor);
                } else if let Some(f) = n.as_f64() {
                    push(slice, cursor, 2);
                    f.convert_to_bytes(slice, cursor);
                }
            }
            Value::String(s) => {
                push(slice, cursor, 3);
                s.convert_to_bytes(slice, cursor);
            }
            Value::Array(a) => {
                push(slice, cursor, 4);
                a.convert_to_bytes(slice, cursor);
            }
            Value::Object(o) => {
                push(slice, cursor, 5);
                let size: u32 = o.len() as u32;
                size.convert_to_bytes(slice, cursor);
                o.iter().for_each(|(k, v)| {
                    k.convert_to_bytes(slice, cursor);
                    v.convert_to_bytes(slice, cursor);
                });
            }
        }
    }

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        match self {
            Value::Null => {
                push(slice, cursor, 0);
            }
            Value::Bool(v) => {
                push(slice, cursor, 1);
                v.convert_to_bytes(slice, cursor);
            }
            Value::Number(n) => {
                push(slice, cursor, 2);
                if let Some(u) = n.as_u64() {
                    push(slice, cursor, 0);
                    u.convert_to_bytes(slice, cursor);
                } else if let Some(i) = n.as_i64() {
                    push(slice, cursor, 1);
                    i.convert_to_bytes(slice, cursor);
                } else if let Some(f) = n.as_f64() {
                    push(slice, cursor, 2);
                    if f.is_infinite() {
                        return Err(RapiraError::FloatIsNaNError);
                    }
                    f.try_convert_to_bytes(slice, cursor)?;
                }
            }
            Value::String(s) => {
                push(slice, cursor, 3);
                s.convert_to_bytes(slice, cursor);
            }
            Value::Array(a) => {
                push(slice, cursor, 4);
                a.try_convert_to_bytes(slice, cursor)?;
            }
            Value::Object(o) => {
                push(slice, cursor, 5);
                let size: u32 = o.len() as u32;
                size.convert_to_bytes(slice, cursor);
                for (k, v) in o.iter() {
                    k.convert_to_bytes(slice, cursor);
                    v.try_convert_to_bytes(slice, cursor)?;
                }
            }
        }
        Ok(())
    }

    #[inline]
    fn size(&self) -> usize {
        1 + match self {
            Value::Null => 0,
            Value::Bool(_) => 1,
            Value::Number(_) => 1 + 8,
            Value::String(s) => s.size(),
            Value::Array(vec) => 4 + vec.iter().fold(0, |acc, item| acc + item.size()),
            Value::Object(v) => {
                4 + v
                    .iter()
                    .fold(0, |acc, item| acc + item.0.size() + item.1.size())
            }
        }
    }
}

#[cfg(feature = "decimal")]
impl Rapira for Decimal {
    const STATIC_SIZE: Option<usize> = Some(16);

    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let bytes = <[u8; 16]>::from_slice_unsafe(slice)?;
        Ok(Decimal::deserialize(bytes))
    }

    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let bytes = <[u8; 16]>::from_slice(slice)?;
        Ok(Decimal::deserialize(bytes))
    }

    fn check_bytes(_: &mut &[u8]) -> Result<()> {
        Ok(())
    }

    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        self.serialize().convert_to_bytes(slice, cursor);
    }

    fn size(&self) -> usize {
        16
    }
}
