use crate::{enum_static_size, push, static_size, Rapira, RapiraError, Result};
use core::{
    mem::{size_of, MaybeUninit},
    num::NonZeroU32,
};

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
    use core::marker::PhantomData;

    use super::*;

    pub const fn static_size<T>(_: PhantomData<T>) -> Option<usize> {
        Some(1)
    }

    #[inline]
    pub fn size(_: &u8) -> usize {
        1
    }

    #[inline]
    pub fn check_bytes<T>(_: PhantomData<T>, slice: &mut &[u8]) -> Result<()> {
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

#[inline(always)]
pub fn into_arr<const N: usize>(slice: &[u8]) -> Result<[u8; N], RapiraError> {
    let slice = slice.get(..N).ok_or(RapiraError::SliceLenError)?;
    let arr: &[u8; N] = unsafe { core::mem::transmute_copy(&slice) };
    Ok(*arr)
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
                let bytes: [u8; size_of::<$type>()] = into_arr(slice)?;
                let u = <$type>::from_le_bytes(bytes);

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

/// as i64
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
        let bytes: [u8; size_of::<Self>()] = into_arr(slice)?;
        let u = f32::from_le_bytes(bytes);

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
        let bytes: [u8; size_of::<Self>()] = into_arr(slice)?;
        let u = f32::from_le_bytes(bytes);

        *slice = unsafe { slice.get_unchecked(size_of::<Self>()..) };
        Ok(u)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        let bytes: [u8; size_of::<Self>()] = into_arr(slice)?;
        let u = Self::from_le_bytes(bytes);

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
        let bytes: [u8; size_of::<Self>()] = into_arr(slice)?;
        let u = f64::from_le_bytes(bytes);

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
        let bytes: [u8; size_of::<Self>()] = into_arr(slice)?;
        let u = f64::from_le_bytes(bytes);

        *slice = unsafe { slice.get_unchecked(size_of::<Self>()..) };
        Ok(u)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        let bytes: [u8; size_of::<Self>()] = into_arr(slice)?;
        let u = Self::from_le_bytes(bytes);

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

impl<T: Rapira> Rapira for Option<T> {
    const STATIC_SIZE: Option<usize> = enum_static_size(T::STATIC_SIZE);

    #[inline]
    fn size(&self) -> usize {
        match self {
            None => 1,
            Some(t) => 1 + t.size(),
        }
    }

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
}

impl<const CAP: usize> Rapira for [u8; CAP] {
    const STATIC_SIZE: Option<usize> = Some(CAP);

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        *slice = slice.get(CAP..).ok_or(RapiraError::SliceLenError)?;
        Ok(())
    }

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let bytes: [u8; CAP] = into_arr(slice)?;

        *slice = unsafe { slice.get_unchecked(CAP..) };
        Ok(bytes)
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
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        let end = *cursor + CAP;
        let s = slice
            .get_mut(*cursor..end)
            .ok_or(RapiraError::SliceLenError)?;
        s.copy_from_slice(self);
        *cursor = end;
        Ok(())
    }

    #[inline]
    fn size(&self) -> usize {
        CAP
    }
}

impl<T, const CAP: usize> Rapira for [T; CAP]
where
    T: Rapira + Sized,
{
    const STATIC_SIZE: Option<usize> = static_size([T::STATIC_SIZE; CAP]);

    #[inline]
    fn size(&self) -> usize {
        match T::STATIC_SIZE {
            Some(size) => size * CAP,
            None => self.iter().map(|i| i.size()).sum(),
        }
    }

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
        let mut arr: [MaybeUninit<T>; CAP] = unsafe { MaybeUninit::uninit().assume_init() };

        for i in 0..CAP {
            match T::from_slice(slice) {
                Ok(val) => {
                    arr.get_mut(i).ok_or(RapiraError::SliceLenError)?.write(val);
                }
                Err(err) => {
                    if i != 0 {
                        let s = arr.get_mut(0..i).ok_or(RapiraError::SliceLenError)?;

                        for item in s {
                            unsafe {
                                item.assume_init_drop();
                            }
                        }
                    }
                    return Err(err);
                }
            }
        }

        let arr: [T; CAP] = arr.map(|i| unsafe { i.assume_init() });

        Ok(arr)
    }

    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let mut arr: [MaybeUninit<T>; CAP] = unsafe { MaybeUninit::uninit().assume_init() };

        for i in 0..CAP {
            match T::from_slice_unchecked(slice) {
                Ok(val) => unsafe {
                    arr.get_unchecked_mut(i).write(val);
                },
                Err(err) => {
                    if i != 0 {
                        unsafe {
                            for item in arr.get_unchecked_mut(0..i) {
                                item.assume_init_drop();
                            }
                        }
                    }
                    return Err(err);
                }
            }
        }

        let arr: [T; CAP] = arr.map(|i| unsafe { i.assume_init() });

        Ok(arr)
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let mut arr: [MaybeUninit<T>; CAP] = MaybeUninit::uninit().assume_init();

        for i in 0..CAP {
            match T::from_slice_unsafe(slice) {
                Ok(val) => {
                    arr.get_unchecked_mut(i).write(val);
                }
                Err(err) => {
                    if i != 0 {
                        for item in arr.get_unchecked_mut(0..i) {
                            item.assume_init_drop();
                        }
                    }
                    return Err(err);
                }
            }
        }

        let arr: [T; CAP] = arr.map(|i| i.assume_init());

        Ok(arr)
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        for val in self.iter() {
            val.convert_to_bytes(slice, cursor);
        }
    }

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        for val in self.iter() {
            val.try_convert_to_bytes(slice, cursor)?;
        }
        Ok(())
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

pub mod str_rapira {
    use core::marker::PhantomData;

    use simdutf8::basic::from_utf8;

    use crate::extend;

    use super::*;

    const STR_MIN_LEN_SIMD: usize = 16;

    pub const fn static_size<T>(_: PhantomData<T>) -> Option<usize> {
        None
    }

    #[inline]
    pub fn size(s: &str) -> usize {
        4 + s.len()
    }

    #[inline]
    pub fn check_bytes<T>(_: PhantomData<T>, slice: &mut &[u8]) -> Result<()> {
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
    use core::marker::PhantomData;

    use crate::extend;

    use super::*;

    pub const fn static_size<T>(_: PhantomData<T>) -> Option<usize> {
        None
    }

    #[inline]
    pub fn size(s: &[u8]) -> usize {
        4 + s.len()
    }

    #[inline]
    pub fn check_bytes<T>(_: PhantomData<T>, slice: &mut &[u8]) -> Result<()> {
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
