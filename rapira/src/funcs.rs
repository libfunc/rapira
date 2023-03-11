use crate::{Rapira, Result};

#[inline]
pub fn size<T: Rapira>(item: &T) -> usize {
    match T::STATIC_SIZE {
        Some(s) => s,
        None => item.size(),
    }
}

#[cfg(feature = "alloc")]
pub fn serialize<T: Rapira>(item: &T) -> Vec<u8> {
    let value_size = size(item);
    let mut bytes: Vec<u8> = vec![0u8; value_size];
    let mut cursor = 0usize;
    item.convert_to_bytes(&mut bytes, &mut cursor);
    bytes
}

#[cfg(feature = "alloc")]
pub fn extend_vec<T: Rapira>(item: &T, bytes: &mut Vec<u8>) {
    let value_size = size(item);
    let mut cursor = bytes.len();
    bytes.resize(cursor + value_size, 0);
    item.convert_to_bytes(bytes, &mut cursor);
}

/// for unsafe data
pub fn check_bytes<T: Rapira>(bytes: &[u8]) -> Result<()>
where
    T: Sized,
{
    let mut bytes = bytes;
    T::check_bytes(&mut bytes)
}

/// call only for safe data, not external data
/// not check oversize vec, string and other items with capacity and len
/// but check utf-8 strings, float numbers, non zero numbers and others...
pub fn deserialize<T: Rapira>(mut bytes: &[u8]) -> Result<T>
where
    T: Sized,
{
    T::from_slice(&mut bytes)
}

pub fn deser_unchecked<T: Rapira>(mut bytes: &[u8]) -> Result<T>
where
    T: Sized,
{
    T::from_slice_unchecked(&mut bytes)
}

/// # Safety
///
/// This is unsafe
pub unsafe fn deser_unsafe<T: Rapira>(mut bytes: &[u8]) -> Result<T>
where
    T: Sized,
{
    T::from_slice_unsafe(&mut bytes)
}
