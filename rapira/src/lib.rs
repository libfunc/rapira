// #![feature(trace_macros)]
// #![feature(log_syntax)]
#![cfg_attr(not(feature = "std"), no_std)]

mod allocated;
pub mod error;
pub mod funcs;
mod implements;
#[cfg(feature = "std")]
mod macros;
pub mod max_cap;
mod primitive;

pub use error::{RapiraError, Result};
#[cfg(feature = "zerocopy")]
pub use implements::zero;
pub use primitive::{byte_rapira, bytes_rapira, str_rapira};

#[cfg(feature = "alloc")]
extern crate alloc;

pub use funcs::{check_bytes, deser_unchecked, deser_unsafe, deserialize, size};
#[cfg(feature = "alloc")]
pub use funcs::{extend_vec, serialize};
pub use rapira_derive::Rapira;

pub trait Rapira {
    const STATIC_SIZE: Option<usize> = None;

    fn size(&self) -> usize;

    /// check bytes, collections len, check utf-8, NonZero, f32 and others...
    fn check_bytes(slice: &mut &[u8]) -> Result<()>;

    /// this is safe, but not check collections capacity!
    /// recommend use only for safe data (example: from DB), not external data.
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized;

    /// this mean not unsafe, but unchecked
    /// utf-8 strings, NonZero, float numbers not check
    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
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
}

#[inline]
pub fn try_push(slice: &mut [u8], cursor: &mut usize, item: u8) -> Result<()> {
    let s = slice.get_mut(*cursor).ok_or(RapiraError::SliceLenError)?;
    *s = item;
    *cursor += 1;

    Ok(())
}

#[inline]
pub fn extend(slice: &mut [u8], cursor: &mut usize, items: &[u8]) {
    let end = *cursor + items.len();
    let s = unsafe { slice.get_unchecked_mut(*cursor..end) };
    s.copy_from_slice(items);
    *cursor = end;
}

#[inline]
pub fn try_extend(slice: &mut [u8], cursor: &mut usize, items: &[u8]) -> Result<()> {
    let end = *cursor + items.len();
    let s = slice
        .get_mut(*cursor..end)
        .ok_or(RapiraError::SliceLenError)?;
    s.copy_from_slice(items);
    *cursor = end;

    Ok(())
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
