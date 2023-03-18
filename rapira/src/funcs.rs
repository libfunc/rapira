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

/// Check oversize vec and other items with capacity initialization
/// (max memory limit attack...)
/// check cursor oveflow,
/// check utf-8 strings, float numbers, non zero numbers and others...
pub fn check_bytes<T: Rapira>(bytes: &[u8]) -> Result<()>
where
    T: Sized,
{
    let mut bytes = bytes;
    T::check_bytes(&mut bytes)
}

/// Check oversize vec and other items with capacity initialization
/// with MaxCapacity trait
/// (max memory limit attack...)
///
/// but check cursor oveflow,
/// and check utf-8 strings, float numbers, non zero numbers and others...
pub fn deserialize<T: Rapira>(mut bytes: &[u8]) -> Result<T>
where
    T: Sized,
{
    T::from_slice(&mut bytes)
}

/// NOT check oversize vec and other items with capacity initialization
/// (max memory limit attack...)
/// NOT check utf-8 strings, float numbers, non zero numbers and others...
///
/// but check cursor oveflow
/// Another way - data maybe not correct, but not read from other memory
pub fn deser_unchecked<T: Rapira>(mut bytes: &[u8]) -> Result<T>
where
    T: Sized,
{
    T::from_slice_unchecked(&mut bytes)
}

/// # Safety
///
/// This is extremally unsafe and UB maybe...
/// Call only after check_bytes fn!
///
/// NOT check oversize vec and other items with capacity initialization
/// (max memory limit attack...)
/// NOT check utf-8 strings, float numbers, non zero numbers and others...
/// NOT check cursor oveflow
pub unsafe fn deser_unsafe<T: Rapira>(mut bytes: &[u8]) -> Result<T>
where
    T: Sized,
{
    T::from_slice_unsafe(&mut bytes)
}
