use crate::{Rapira, RapiraError, Result};
use core::{mem::size_of, num::NonZeroU32};

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
        let byte = unsafe { slice.get_unchecked_mut(*cursor) };
        *byte = u8::from(*self);
        *cursor += 1;
    }

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        let byte = slice.get_mut(*cursor).ok_or(RapiraError::SliceLenError)?;
        *byte = u8::from(*self);
        *cursor += 1;
        Ok(())
    }

    #[inline]
    fn size(&self) -> usize {
        1
    }
}

pub mod byte_rapira {
    use super::*;

    #[allow(clippy::extra_unused_type_parameters)]
    pub const fn static_size<T>() -> Option<usize> {
        Some(1)
    }

    #[inline]
    pub fn size(_: &u8) -> usize {
        1
    }

    #[allow(clippy::extra_unused_type_parameters)]
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
        let byte = slice.get_mut(*cursor).ok_or(RapiraError::SliceLenError)?;
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
                    .try_into()?;
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
            fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
                let bytes = self.to_le_bytes();
                let end = *cursor + size_of::<$type>();
                let s = slice
                    .get_mut(*cursor..end)
                    .ok_or(RapiraError::SliceLenError)?;
                s.copy_from_slice(&bytes);
                *cursor = end;
                Ok(())
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

/// as u32
impl Rapira for usize {
    const STATIC_SIZE: Option<usize> = Some(size_of::<u32>());

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        u32::from_slice(slice).map(|u| u as usize)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        u32::check_bytes(slice)
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        u32::from_slice_unsafe(slice).map(|u| u as usize)
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        (*self as u32).convert_to_bytes(slice, cursor);
    }

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        let bytes = (*self as u32).to_le_bytes();
        let end = *cursor + size_of::<u32>();
        let s = slice
            .get_mut(*cursor..end)
            .ok_or(RapiraError::SliceLenError)?;
        s.copy_from_slice(&bytes);
        *cursor = end;
        Ok(())
    }

    #[inline]
    fn size(&self) -> usize {
        size_of::<u32>()
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
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        let bytes = (*self as i64).to_le_bytes();
        let end = *cursor + size_of::<i64>();
        let s = slice
            .get_mut(*cursor..end)
            .ok_or(RapiraError::SliceLenError)?;
        s.copy_from_slice(&bytes);
        *cursor = end;
        Ok(())
    }

    #[inline]
    fn size(&self) -> usize {
        size_of::<i64>()
    }
}

impl Rapira for NonZeroU32 {
    const STATIC_SIZE: Option<usize> = Some(size_of::<Self>());

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        let bytes = slice
            .get(..size_of::<Self>())
            .ok_or(RapiraError::SliceLenError)?;

        if bytes == [0u8; size_of::<Self>()] {
            Err(RapiraError::NonZeroError)
        } else {
            *slice = unsafe { slice.get_unchecked(size_of::<Self>()..) };
            Ok(())
        }
    }

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
    fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let u = u32::from_slice(slice)?;
        let u = unsafe { NonZeroU32::new_unchecked(u) };

        *slice = unsafe { slice.get_unchecked(size_of::<Self>()..) };
        Ok(u)
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
            .try_into()?;
        let u = f32::from_le_bytes(*bytes);

        if !u.is_finite() {
            return Err(RapiraError::FloatIsNaNError);
        }

        *slice = unsafe { slice.get_unchecked(size_of::<Self>()..) };
        Ok(u)
    }

    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let bytes: &[u8; size_of::<Self>()] = slice
            .get(..size_of::<Self>())
            .ok_or(RapiraError::SliceLenError)?
            .try_into()?;
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
            .try_into()?;
        let u = Self::from_le_bytes(*bytes);

        if !u.is_finite() {
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
        if !self.is_finite() {
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
            .try_into()?;
        let u = f64::from_le_bytes(*bytes);

        if !u.is_finite() {
            return Err(RapiraError::FloatIsNaNError);
        }

        *slice = unsafe { slice.get_unchecked(size_of::<Self>()..) };
        Ok(u)
    }

    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let bytes: &[u8; size_of::<Self>()] = slice
            .get(..size_of::<Self>())
            .ok_or(RapiraError::SliceLenError)?
            .try_into()?;
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
            .try_into()?;
        let u = Self::from_le_bytes(*bytes);

        if !u.is_finite() {
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
        if !self.is_finite() {
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
        *cursor = end;
    }

    #[inline]
    fn size(&self) -> usize {
        8
    }
}

// impl Rapira for &'static str {
//     #[inline]
//     fn check_bytes(slice: &mut &[u8]) -> Result<()>
//     where
//         Self: Sized,
//     {
//         let len = u32::from_slice(slice)? as usize;
//         let bytes = slice.get(..len).ok_or(RapiraError::SliceLenError)?;
//         let _ = str_from_bytes(bytes)?;
//         *slice = unsafe { slice.get_unchecked(len..) };
//         Ok(())
//     }
//     #[inline]
//     fn from_slice(slice: &mut &[u8]) -> Result<Self>
//     where
//         Self: Sized,
//     {
//         let len = u32::from_slice(slice)? as usize;
//         let bytes = slice.get(..len).ok_or(RapiraError::SliceLenError)?;
//         let _s = if len > STR_MIN_LEN_SIMD {
//             from_utf8(bytes).map_err(|_| RapiraError::StringTypeError)?
//         } else {
//             core::str::from_utf8(bytes).map_err(|_| RapiraError::StringTypeError)?
//         };
//         *slice = unsafe { slice.get_unchecked(len..) };
//         todo!()
//     }
//     #[inline]
//     fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
//     where
//         Self: Sized,
//     {
//         let len = u32::from_slice(slice)? as usize;
//         let bytes = slice.get(..len).ok_or(RapiraError::SliceLenError)?;
//         let _s = unsafe { core::str::from_utf8_unchecked(bytes) };
//         *slice = unsafe { slice.get_unchecked(len..) };
//         todo!()
//     }
//     #[inline]
//     unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
//     where
//         Self: Sized,
//     {
//         let len = get_u32_unsafe(slice) as usize;
//         let bytes = slice.get_unchecked(..len);
//         let _s = core::str::from_utf8_unchecked(bytes);
//         *slice = slice.get_unchecked(len..);
//         todo!()
//     }
//     #[inline]
//     fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
//         let len = self.len() as u32;
//         len.convert_to_bytes(slice, cursor);
//         extend(slice, cursor, self.as_bytes());
//     }
//     #[inline]
//     fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
//         let len = self.len() as u32;
//         len.try_convert_to_bytes(slice, cursor)?;
//         extend(slice, cursor, self.as_bytes());
//         Ok(())
//     }
//     #[inline]
//     fn size(&self) -> usize {
//         4 + self.len()
//     }
// }

pub mod str_rapira {
    use simdutf8::basic::from_utf8;

    use crate::extend;

    use super::*;

    const STR_MIN_LEN_SIMD: usize = 16;

    #[allow(clippy::extra_unused_type_parameters)]
    pub const fn static_size<T>() -> Option<usize> {
        None
    }

    #[inline]
    pub fn size(s: &str) -> usize {
        4 + s.len()
    }

    #[allow(clippy::extra_unused_type_parameters)]
    #[inline]
    pub fn check_bytes<T>(slice: &mut &[u8]) -> Result<()> {
        let len = u32::from_slice(slice)? as usize;
        let bytes = slice.get(..len).ok_or(RapiraError::SliceLenError)?;

        if len > STR_MIN_LEN_SIMD {
            from_utf8(bytes).map_err(|_| RapiraError::StringTypeError)?;
        } else {
            core::str::from_utf8(bytes).map_err(|_| RapiraError::StringTypeError)?;
        };

        *slice = unsafe { slice.get_unchecked(len..) };
        Ok(())
    }

    #[inline]
    pub fn from_slice<'a>(slice: &mut &'a [u8]) -> Result<&'a str> {
        let len = u32::from_slice(slice)? as usize;
        let bytes = slice.get(..len).ok_or(RapiraError::SliceLenError)?;
        let s = if len > STR_MIN_LEN_SIMD {
            from_utf8(bytes).map_err(|_| RapiraError::StringTypeError)?
        } else {
            core::str::from_utf8(bytes).map_err(|_| RapiraError::StringTypeError)?
        };

        *slice = unsafe { slice.get_unchecked(len..) };
        Ok(s)
    }

    #[inline]
    pub fn from_slice_unchecked<'a>(slice: &mut &'a [u8]) -> Result<&'a str> {
        let len = u32::from_slice(slice)? as usize;
        let bytes = slice.get(..len).ok_or(RapiraError::SliceLenError)?;
        let s = unsafe { core::str::from_utf8_unchecked(bytes) };

        *slice = unsafe { slice.get_unchecked(len..) };
        Ok(s)
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
    pub unsafe fn from_slice_unsafe<'a>(slice: &mut &'a [u8]) -> Result<&'a str> {
        let len = usize::from_slice_unsafe(slice)?;
        let bytes = slice.get_unchecked(..len);

        let s = core::str::from_utf8_unchecked(bytes);

        *slice = slice.get_unchecked(len..);
        Ok(s)
    }

    #[inline]
    pub fn convert_to_bytes(item: &str, slice: &mut [u8], cursor: &mut usize) {
        let len = item.len() as u32;
        len.convert_to_bytes(slice, cursor);
        extend(slice, cursor, item.as_bytes());
    }

    #[inline]
    pub fn try_convert_to_bytes(item: &str, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        let len = item.len() as u32;
        len.try_convert_to_bytes(slice, cursor)?;
        extend(slice, cursor, item.as_bytes());

        Ok(())
    }
}

pub mod bytes_rapira {
    use crate::extend;

    use super::*;

    #[allow(clippy::extra_unused_type_parameters)]
    pub const fn static_size<T>() -> Option<usize> {
        None
    }

    #[inline]
    pub fn size(s: &[u8]) -> usize {
        4 + s.len()
    }

    #[allow(clippy::extra_unused_type_parameters)]
    #[inline]
    pub fn check_bytes<T>(slice: &mut &[u8]) -> Result<()> {
        let len = u32::from_slice(slice)? as usize;
        *slice = slice.get(len..).ok_or(RapiraError::SliceLenError)?;
        Ok(())
    }

    #[inline]
    pub fn from_slice<'a>(slice: &mut &'a [u8]) -> Result<&'a [u8]> {
        let len = u32::from_slice(slice)? as usize;
        let bytes = slice.get(..len).ok_or(RapiraError::SliceLenError)?;
        *slice = unsafe { slice.get_unchecked(len..) };
        Ok(bytes)
    }

    #[inline]
    pub fn from_slice_unchecked<'a>(slice: &mut &'a [u8]) -> Result<&'a [u8]> {
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
    pub unsafe fn from_slice_unsafe<'a>(slice: &mut &'a [u8]) -> Result<&'a [u8]> {
        let len = usize::from_slice_unsafe(slice)?;
        let bytes = slice.get_unchecked(..len);
        *slice = slice.get_unchecked(len..);
        Ok(bytes)
    }

    #[inline]
    pub fn convert_to_bytes(item: &[u8], slice: &mut [u8], cursor: &mut usize) {
        let len = item.len() as u32;
        len.convert_to_bytes(slice, cursor);
        extend(slice, cursor, item);
    }

    #[inline]
    pub fn try_convert_to_bytes(item: &[u8], slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        let len = item.len() as u32;
        len.try_convert_to_bytes(slice, cursor)?;
        extend(slice, cursor, item);
        Ok(())
    }
}
