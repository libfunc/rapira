use crate::{extend, get_u32_unsafe, Rapira, RapiraError, Result};
use simdutf8::basic::from_utf8;

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, collections::BTreeMap, string::String, vec::Vec};

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
