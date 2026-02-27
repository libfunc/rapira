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
#[cfg(feature = "postcard")]
pub use implements::postcard;
#[cfg(feature = "zerocopy")]
pub use implements::zero;
pub use primitive::{byte_rapira, bytes_rapira, str_rapira};

#[cfg(feature = "alloc")]
extern crate alloc;

pub use funcs::{
    check_bytes, deser_unchecked, deser_unsafe, deserialize, deserialize_versioned, size,
};
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

    #[inline]
    fn debug_from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized + std::fmt::Debug,
    {
        let len = slice.len();
        Self::from_slice(slice)
            .inspect(|item| {
                println!("len: {len}, item: {item:?}");
            })
            .inspect_err(|err| {
                println!("len: {len}, err: {err:?}");
            })
    }

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

    /// Deserialize with schema version awareness.
    ///
    /// Enables backward-compatible deserialization: new code can read old data
    /// that is missing fields added in later versions. The version number is
    /// stored **externally** (e.g. in DB metadata), not inside the serialized bytes.
    ///
    /// Default implementation delegates to [`from_slice`](Rapira::from_slice).
    /// The derive macro generates an override when `#[rapira(version = N)]` is
    /// placed on a struct — fields annotated with `#[rapira(since = M)]` are
    /// only read from the slice when `version >= M`, otherwise
    /// [`Default::default()`] is used.
    ///
    /// # Derive usage
    ///
    /// ```rust,ignore
    /// #[derive(Rapira)]
    /// #[rapira(version = 2)]
    /// struct User {
    ///     name: String,           // present since v1 (no attribute needed)
    ///     age: u32,               // present since v1
    ///     #[rapira(since = 2)]
    ///     email: Option<String>,  // added in v2, defaults to None for v1 data
    /// }
    /// ```
    ///
    /// # Reading old data
    ///
    /// ```rust,ignore
    /// // version comes from DB metadata, not from the bytes themselves
    /// let user: User = rapira::deserialize_versioned(&bytes, schema_version)?;
    /// ```
    ///
    /// # Rules
    ///
    /// - Serialization (`convert_to_bytes`, `size`) always writes **all** fields
    ///   (current version). Only deserialization is affected.
    /// - `from_slice` always reads all fields regardless of version (use for
    ///   current-version data).
    /// - Fields with `#[rapira(since = M)]` must implement [`Default`].
    /// - `since = 0` is invalid (versions start at 1).
    /// - `since` value must not exceed the struct's `version`.
    /// - `#[rapira(since)]` requires `#[rapira(version)]` on the struct.
    /// - `#[rapira(since)]` and `#[rapira(skip)]` cannot be combined.
    /// - Structs without `#[rapira(version)]` use the default impl (delegates
    ///   to `from_slice`), so existing code is unaffected.
    /// - Version is propagated through collections (`Vec<T>`, `Option<T>`,
    ///   `Box<T>`, `BTreeMap`, tuples, arrays, `SmallVec`, `ArrayVec`).
    #[inline]
    fn from_slice_versioned(slice: &mut &[u8], _version: u8) -> Result<Self>
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
