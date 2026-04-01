use crate::{Rapira, RapiraFlags, Result};

#[inline]
pub fn size<T: Rapira>(item: &T) -> usize {
    match T::STATIC_SIZE {
        Some(s) => s,
        None => item.size(),
    }
}

pub fn write_to_array<const N: usize, T: Rapira>(item: &T, bytes: &mut [u8; N]) {
    assert_eq!(N, T::STATIC_SIZE.unwrap());
    item.convert_to_bytes(bytes, &mut 0);
}

/// serialize obect and return vec of bytes
#[cfg(feature = "alloc")]
pub fn serialize<T: Rapira>(item: &T) -> Vec<u8> {
    let value_size = size(item);
    let mut bytes: Vec<u8> = vec![0u8; value_size];
    item.convert_to_bytes(&mut bytes, &mut 0);
    bytes
}

/// extend vec of bytes with serialized object
#[cfg(feature = "alloc")]
pub fn extend_vec<T: Rapira>(item: &T, bytes: &mut Vec<u8>) {
    let value_size = size(item);
    let mut cursor = bytes.len();
    bytes.resize(cursor + value_size, 0);
    item.convert_to_bytes(bytes, &mut cursor);
}

/// write to vec of bytes with serialized object
#[cfg(feature = "alloc")]
pub fn write_to_vec<T: Rapira>(item: &T, bytes: &mut Vec<u8>) {
    let value_size = size(item);
    let len = bytes.len();
    if len < value_size {
        bytes.resize(value_size, 0);
    }
    item.convert_to_bytes(bytes, &mut 0);
}

/// Check oversize vec and other items with capacity initialization
/// (max memory limit attack...)
/// check cursor oveflow,
/// check utf-8 strings, float numbers, non zero numbers and others...
pub fn check_bytes<T>(bytes: &[u8]) -> Result<()>
where
    T: Rapira + Sized,
{
    let mut bytes = bytes;
    T::check_bytes(&mut bytes)
}

/// use `deserialize` function to read data to slice
pub fn read_to<T: Rapira>(mut bytes: &[u8], mut iter: impl Extend<T>) -> Result<()> {
    while !bytes.is_empty() {
        let item = T::from_slice(&mut bytes)?;
        iter.extend(Some(item));
    }
    Ok(())
}

// #[cfg(feature = "std")]
// pub fn encode_to_writer(item: &impl Rapira, mut writer: impl std::io::Write) -> Result<()> {
//     let value_size = size(item);
//     let mut bytes = vec![0u8; value_size];
//     item.convert_to_bytes(&mut bytes, &mut 0);
//     writer.write_all(&bytes).map_err(crate::RapiraError::from)?;
//     Ok(())
// }

// #[cfg(feature = "std")]
// pub fn decode_from_reader<T: Rapira>(mut reader: impl std::io::Read) -> Result<T> {
//     let mut bytes = Vec::new();
//     reader
//         .read_to_end(&mut bytes)
//         .map_err(crate::RapiraError::from)?;
//     deserialize(&bytes)
// }

/// Check oversize vec and other items with capacity initialization
/// with MaxCapacity trait
/// (max memory limit attack...)
///
/// Check cursor oveflow,
/// and check utf-8 strings, float numbers, non zero numbers and others...
pub fn deserialize<T>(mut bytes: &[u8]) -> Result<T>
where
    T: Rapira + Sized,
{
    T::from_slice(&mut bytes)
}

/// Deserialize with schema version awareness.
/// Version is stored externally (e.g. in DB metadata), not in the serialized data.
pub fn deserialize_versioned<T>(mut bytes: &[u8], version: u8) -> Result<T>
where
    T: Rapira + Sized,
{
    T::from_slice_versioned(&mut bytes, version)
}

/// # Safety
///
/// NOT check oversize vec and other items with capacity initialization
/// (max memory limit attack...)
/// NOT check utf-8 strings, float numbers, non zero numbers,
/// Arrayvec len and others...
///
/// but check cursor oveflow
/// Another way - data maybe not correct, but not read from other memory
pub unsafe fn deser_unchecked<T>(mut bytes: &[u8]) -> Result<T>
where
    T: Rapira + Sized,
{
    unsafe { T::from_slice_unchecked(&mut bytes) }
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
pub unsafe fn deser_unsafe<T>(mut bytes: &[u8]) -> Result<T>
where
    T: Rapira + Sized,
{
    unsafe { T::from_slice_unsafe(&mut bytes) }
}

#[inline]
pub fn size_ctx<T: Rapira>(item: &T, flags: RapiraFlags) -> usize {
    match T::STATIC_SIZE {
        Some(s) => s,
        None => item.size_ctx(flags),
    }
}

/// Serialize with context flags.
#[cfg(feature = "alloc")]
pub fn serialize_ctx<T: Rapira>(item: &T, flags: RapiraFlags) -> Vec<u8> {
    let value_size = size_ctx(item, flags);
    let mut bytes: Vec<u8> = vec![0u8; value_size];
    item.convert_to_bytes_ctx(&mut bytes, &mut 0, flags);
    bytes
}

/// Deserialize with context flags.
pub fn deserialize_ctx<T: Rapira + Sized>(mut bytes: &[u8], flags: RapiraFlags) -> Result<T> {
    T::from_slice_ctx(&mut bytes, flags)
}
