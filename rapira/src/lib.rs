// #![feature(trace_macros)]
// #![feature(log_syntax)]
#![cfg_attr(not(feature = "std"), no_std)]

mod allocated;
pub mod error;
mod from_u8;
pub mod funcs;
mod implements;
#[cfg(feature = "std")]
mod macros;
pub mod max_cap;
mod primitive;

pub use error::{RapiraError, Result};
pub use from_u8::{EnumFromU8Error, FromU8};
#[cfg(feature = "zerocopy")]
pub use implements::zero;
pub use primitive::{byte_rapira, bytes_rapira, str_rapira};

#[cfg(feature = "alloc")]
extern crate alloc;

pub use funcs::{check_bytes, deser_unchecked, deser_unsafe, deserialize, size};
#[cfg(feature = "alloc")]
pub use funcs::{extend_vec, serialize};
pub use rapira_derive::{FromU8, PrimitiveFromEnum, Rapira};

pub trait Rapira {
    const STATIC_SIZE: Option<usize> = None;
    const MIN_SIZE: usize;

    /// size of bytes for serialize
    fn size(&self) -> usize;

    /// check bytes, collections len, check utf-8, NonZero, f32 and others...
    fn check_bytes(slice: &mut &[u8]) -> Result<()>;

    /// this is safe, but not check collections capacity!
    /// recommend use only for safe data (example: from DB), not external data.
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized;

    /// # Safety
    ///
    /// this mean not unsafe, but unchecked
    /// utf-8 strings, NonZero, float numbers not check
    #[inline]
    unsafe fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        Self::from_slice(slice)
    }

    /// # Safety
    ///
    /// This is unsafe, but maybe safe after check_bytes fn
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        Self::from_slice(slice)
    }

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        self.convert_to_bytes(slice, cursor);
        Ok(())
    }

    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize);
}

pub const LEN_SIZE: usize = 4;

#[inline]
pub fn push(slice: &mut [u8], cursor: &mut usize, item: u8) {
    let s = slice.get_mut(*cursor).unwrap();
    *s = item;
    *cursor += 1;
}

#[inline]
pub fn try_push(slice: &mut [u8], cursor: &mut usize, item: u8) -> Result<()> {
    let s = slice.get_mut(*cursor).ok_or(RapiraError::SliceLen)?;
    *s = item;
    *cursor += 1;

    Ok(())
}

#[inline]
pub fn extend(slice: &mut [u8], cursor: &mut usize, items: &[u8]) {
    let end = *cursor + items.len();
    let s = slice.get_mut(*cursor..end).unwrap();
    s.copy_from_slice(items);
    *cursor = end;
}

#[inline]
pub fn try_extend(slice: &mut [u8], cursor: &mut usize, items: &[u8]) -> Result<()> {
    let end = *cursor + items.len();
    let s = slice.get_mut(*cursor..end).ok_or(RapiraError::SliceLen)?;
    s.copy_from_slice(items);
    *cursor = end;

    Ok(())
}

pub const fn static_size<const N: usize>(arr: [Option<usize>; N]) -> Option<usize> {
    let mut i = 0;
    let mut size = 0;
    while i < arr.len() {
        let item = arr[i];
        match item {
            Some(s) => {
                size += s;
            }
            None => {
                return None;
            }
        }
        i += 1;
    }
    Some(size)
}

pub const fn enum_size<const N: usize>(arr: [Option<usize>; N]) -> Option<usize> {
    let mut i = 0;
    let mut size = 0;
    let mut is_init = false;
    while i < arr.len() {
        let item = arr[i];
        match item {
            Some(s) => {
                if !is_init {
                    size = s;
                    is_init = true;
                } else if s != size {
                    return None;
                }
            }
            None => {
                return None;
            }
        }
        i += 1;
    }
    Some(size + 1)
}

pub const fn min_size(arr: &'static [usize]) -> usize {
    let mut i = 0;
    let mut size = 0;
    while i < arr.len() {
        let item = arr[i];
        size += item;
        i += 1;
    }
    size
}

pub const fn enum_min_size(arr: &'static [usize]) -> usize {
    let mut i = 0;
    let mut size = 0;
    let mut is_init = false;
    while i < arr.len() {
        let item = arr[i];
        if !is_init {
            size = item;
            is_init = true;
        } else if size < item {
            size = item;
        }
        i += 1;
    }
    size + 1
}
