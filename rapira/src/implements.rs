#[cfg(feature = "arrayvec")]
use arrayvec::{ArrayString, ArrayVec};
#[cfg(feature = "bytes")]
use bytes::Bytes;
#[cfg(feature = "compact_str")]
use compact_str::CompactString;
#[cfg(feature = "indexmap")]
use core::hash::BuildHasherDefault;
#[cfg(feature = "indexmap")]
use indexmap::IndexMap;
#[cfg(feature = "inline-array")]
use inline_array::InlineArray;
#[cfg(feature = "rust_decimal")]
use rust_decimal::Decimal;
#[cfg(feature = "serde_json")]
use serde_json::{Map, Number, Value};
#[cfg(feature = "smallvec")]
use smallvec::SmallVec;
#[cfg(feature = "uuid")]
use uuid::Uuid;

#[cfg(feature = "smallvec")]
use crate::max_cap::{SMALLVEC_MAX_CAP, SMALLVEC_MAX_SIZE_OF};
#[cfg(feature = "indexmap")]
use crate::max_cap::{VEC_MAX_CAP, VEC_MAX_SIZE_OF};
#[cfg(feature = "arrayvec")]
use crate::str_rapira;

#[cfg(feature = "arrayvec")]
impl<T: crate::Rapira, const CAP: usize> crate::Rapira for ArrayVec<T, CAP> {
    #[inline]
    fn size(&self) -> usize {
        match T::STATIC_SIZE {
            Some(size) => 4 + (size * self.len()),
            None => 4 + self.iter().fold(0, |b, v| b + v.size()),
        }
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> crate::Result<()>
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
    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let len = usize::from_slice(slice)?;

        if len > CAP {
            return Err(crate::RapiraError::SliceLenError);
        }

        let mut vec: ArrayVec<T, CAP> = ArrayVec::new_const();

        for _ in 0..len {
            let val = T::from_slice(slice)?;
            vec.push(val);
        }

        Ok(vec)
    }

    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice_unchecked(slice)? as usize;

        if len > CAP {
            return Err(crate::RapiraError::SliceLenError);
        }

        let mut vec: ArrayVec<T, CAP> = ArrayVec::new_const();

        for _ in 0..len {
            let val = T::from_slice_unchecked(slice)?;
            unsafe {
                vec.push_unchecked(val);
            }
        }

        Ok(vec)
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let len = usize::from_slice_unsafe(slice)?;
        let mut vec: ArrayVec<T, CAP> = ArrayVec::new_const();

        for _ in 0..len {
            let val = T::from_slice_unsafe(slice)?;
            vec.push_unchecked(val);
        }

        Ok(vec)
    }

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> crate::Result<()> {
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
}

#[cfg(feature = "arrayvec")]
impl<const CAP: usize> crate::Rapira for ArrayString<CAP> {
    #[inline]
    fn size(&self) -> usize {
        4 + self.len()
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> crate::Result<()>
    where
        Self: Sized,
    {
        let len = slice.len();
        str_rapira::check_bytes::<()>(core::marker::PhantomData, slice)?;
        let size = len - slice.len();
        if size > CAP {
            Err(crate::RapiraError::SliceLenError)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let s = str_rapira::from_slice(slice)?;
        let s = ArrayString::from(s).map_err(|_| crate::RapiraError::SliceLenError)?;
        Ok(s)
    }

    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let s = str_rapira::from_slice_unchecked(slice)?;
        let s = ArrayString::from(s).map_err(|_| crate::RapiraError::SliceLenError)?;
        Ok(s)
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let s = str_rapira::from_slice_unsafe(slice)?;
        let s = ArrayString::from(s).map_err(|_| crate::RapiraError::SliceLenError)?;
        Ok(s)
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        str_rapira::convert_to_bytes(self, slice, cursor);
    }

    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> crate::Result<()> {
        str_rapira::try_convert_to_bytes(self, slice, cursor)
    }
}

#[cfg(feature = "smallvec")]
impl<T: crate::Rapira, const CAP: usize> crate::Rapira for SmallVec<[T; CAP]> {
    #[inline]
    fn size(&self) -> usize {
        match T::STATIC_SIZE {
            Some(size) => 4 + (size * self.len()),
            None => 4 + self.iter().fold(0, |b, v| b + v.size()),
        }
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> crate::Result<()>
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
    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;

        if len > SMALLVEC_MAX_CAP {
            return Err(crate::RapiraError::MaxCapacity);
        }

        let size = std::mem::size_of::<SmallVec<[T; CAP]>>() * len;

        if size > SMALLVEC_MAX_SIZE_OF {
            return Err(crate::RapiraError::MaxSize);
        }

        let mut vec: SmallVec<[T; CAP]> = SmallVec::with_capacity(len);

        for _ in 0..len {
            let val = T::from_slice(slice)?;
            vec.push(val);
        }

        Ok(vec)
    }

    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;
        let mut vec: SmallVec<[T; CAP]> = SmallVec::with_capacity(len);

        for _ in 0..len {
            let val = T::from_slice_unchecked(slice)?;
            vec.push(val);
        }

        Ok(vec)
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let len = usize::from_slice_unsafe(slice)?;
        let mut vec: SmallVec<[T; CAP]> = SmallVec::with_capacity(len);

        for _ in 0..len {
            let val = T::from_slice_unsafe(slice)?;
            vec.push(val);
        }

        Ok(vec)
    }

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> crate::Result<()> {
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
}

#[cfg(feature = "bytes")]
impl crate::Rapira for Bytes {
    #[inline]
    fn size(&self) -> usize {
        use crate::bytes_rapira;

        bytes_rapira::size(self)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> crate::Result<()> {
        use crate::bytes_rapira;

        bytes_rapira::check_bytes::<()>(core::marker::PhantomData, slice)
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        use crate::bytes_rapira;

        bytes_rapira::convert_to_bytes(self, slice, cursor);
    }

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use crate::bytes_rapira;

        let bytes = bytes_rapira::from_slice(slice)?;
        Ok(Bytes::copy_from_slice(bytes))
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use crate::bytes_rapira;

        let bytes = bytes_rapira::from_slice_unsafe(slice)?;
        Ok(Bytes::copy_from_slice(bytes))
    }
}

#[cfg(feature = "inline-array")]
impl crate::Rapira for InlineArray {
    #[inline]
    fn size(&self) -> usize {
        use crate::bytes_rapira;

        bytes_rapira::size(self)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> crate::Result<()> {
        use crate::bytes_rapira;

        bytes_rapira::check_bytes::<()>(core::marker::PhantomData, slice)
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        use crate::bytes_rapira;

        bytes_rapira::convert_to_bytes(self, slice, cursor);
    }

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use crate::bytes_rapira;

        let bytes = bytes_rapira::from_slice(slice)?;
        Ok(InlineArray::from(bytes))
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use crate::bytes_rapira;

        let bytes = bytes_rapira::from_slice_unsafe(slice)?;
        Ok(InlineArray::from(bytes))
    }
}

#[cfg(feature = "zerocopy")]
pub mod zero {
    use core::{marker::PhantomData, mem::size_of};

    use crate::{extend, try_extend};

    use zerocopy::{AsBytes, FromBytes};

    pub const fn static_size<T>(_: PhantomData<T>) -> Option<usize>
    where
        T: FromBytes + AsBytes + Sized,
    {
        Some(size_of::<T>())
    }

    #[inline]
    pub fn size<T>(_: &T) -> usize
    where
        T: FromBytes + AsBytes + Sized,
    {
        size_of::<T>()
    }

    #[inline]
    pub fn check_bytes<T>(_: PhantomData<T>, slice: &mut &[u8]) -> crate::Result<()>
    where
        T: FromBytes + AsBytes + Sized,
    {
        let size = size_of::<T>();

        *slice = slice.get(size..).ok_or(crate::RapiraError::SliceLenError)?;

        Ok(())
    }

    #[inline]
    pub fn from_slice<T>(slice: &mut &[u8]) -> crate::Result<T>
    where
        T: FromBytes + AsBytes + Sized,
    {
        let size = size_of::<T>();
        let bytes: &[u8] = slice.get(..size).ok_or(crate::RapiraError::SliceLenError)?;

        *slice = unsafe { slice.get_unchecked(size..) };

        let t: T =
            FromBytes::read_from(bytes).ok_or(crate::RapiraError::OtherError("zerocopy error"))?;
        Ok(t)
    }

    #[inline]
    pub fn from_slice_unchecked<T>(slice: &mut &[u8]) -> crate::Result<T>
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
    pub unsafe fn from_slice_unsafe<T>(slice: &mut &[u8]) -> crate::Result<T>
    where
        T: FromBytes + AsBytes + Sized,
    {
        let size = size_of::<T>();
        let bytes: &[u8] = slice.get_unchecked(..size);

        *slice = slice.get_unchecked(size..);

        let t: T =
            FromBytes::read_from(bytes).ok_or(crate::RapiraError::OtherError("zerocopy error"))?;
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
    pub fn try_convert_to_bytes<T>(
        item: &T,
        slice: &mut [u8],
        cursor: &mut usize,
    ) -> crate::Result<()>
    where
        T: FromBytes + AsBytes + Sized,
    {
        let bytes = item.as_bytes();
        try_extend(slice, cursor, bytes)?;
        Ok(())
    }
}

#[cfg(feature = "serde_json")]
impl crate::Rapira for Value {
    #[inline]
    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use crate::byte_rapira;

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
                    let number = Number::from_f64(f).ok_or(crate::RapiraError::FloatIsNaNError)?;
                    Ok(Value::Number(number))
                } else {
                    Err(crate::RapiraError::EnumVariantError)
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
            _ => Err(crate::RapiraError::EnumVariantError),
        }
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> crate::Result<()>
    where
        Self: Sized,
    {
        use crate::byte_rapira;

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
                    return Err(crate::RapiraError::EnumVariantError);
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
            _ => return Err(crate::RapiraError::EnumVariantError),
        }

        Ok(())
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use crate::byte_rapira;

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
                    let number = Number::from_f64(f).ok_or(crate::RapiraError::FloatIsNaNError)?;
                    Ok(Value::Number(number))
                } else {
                    Err(crate::RapiraError::EnumVariantError)
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
                let len = usize::from_slice_unsafe(slice)?;
                let mut map = Map::new();
                for _ in 0..len {
                    let key = String::from_slice_unsafe(slice)?;
                    let val = Value::from_slice_unsafe(slice)?;
                    map.insert(key, val);
                }
                Ok(Value::Object(map))
            }
            _ => Err(crate::RapiraError::EnumVariantError),
        }
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        use crate::push;

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
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> crate::Result<()> {
        use crate::try_push;

        match self {
            Value::Null => {
                try_push(slice, cursor, 0)?;
            }
            Value::Bool(v) => {
                try_push(slice, cursor, 1)?;
                v.convert_to_bytes(slice, cursor);
            }
            Value::Number(n) => {
                try_push(slice, cursor, 2)?;
                if let Some(u) = n.as_u64() {
                    try_push(slice, cursor, 0)?;
                    u.convert_to_bytes(slice, cursor);
                } else if let Some(i) = n.as_i64() {
                    try_push(slice, cursor, 1)?;
                    i.convert_to_bytes(slice, cursor);
                } else if let Some(f) = n.as_f64() {
                    try_push(slice, cursor, 2)?;
                    if f.is_infinite() {
                        return Err(crate::RapiraError::FloatIsNaNError);
                    }
                    f.try_convert_to_bytes(slice, cursor)?;
                }
            }
            Value::String(s) => {
                try_push(slice, cursor, 3)?;
                s.convert_to_bytes(slice, cursor);
            }
            Value::Array(a) => {
                try_push(slice, cursor, 4)?;
                a.try_convert_to_bytes(slice, cursor)?;
            }
            Value::Object(o) => {
                try_push(slice, cursor, 5)?;
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

#[cfg(feature = "rust_decimal")]
impl crate::Rapira for Decimal {
    const STATIC_SIZE: Option<usize> = Some(16);

    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let bytes = <[u8; 16]>::from_slice_unsafe(slice)?;
        Ok(Decimal::deserialize(bytes))
    }

    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let bytes = <[u8; 16]>::from_slice(slice)?;
        Ok(Decimal::deserialize(bytes))
    }

    fn check_bytes(_: &mut &[u8]) -> crate::Result<()> {
        Ok(())
    }

    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        self.serialize().convert_to_bytes(slice, cursor);
    }

    fn size(&self) -> usize {
        16
    }
}

#[cfg(feature = "compact_str")]
impl crate::Rapira for CompactString {
    fn size(&self) -> usize {
        4 + self.len()
    }

    fn check_bytes(slice: &mut &[u8]) -> crate::Result<()> {
        str_rapira::check_bytes::<()>(core::marker::PhantomData, slice)
    }

    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let s = str_rapira::from_slice(slice)?;
        let s = CompactString::new(s);
        Ok(s)
    }

    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let s = str_rapira::from_slice_unsafe(slice)?;
        let s = CompactString::new(s);
        Ok(s)
    }

    fn from_slice_unchecked(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let s = str_rapira::from_slice_unchecked(slice)?;
        let s = CompactString::new(s);
        Ok(s)
    }

    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        str_rapira::convert_to_bytes(self, slice, cursor);
    }

    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> crate::Result<()> {
        str_rapira::try_convert_to_bytes(self, slice, cursor)
    }
}

#[cfg(feature = "indexmap")]
impl<K: crate::Rapira, V: crate::Rapira, S> crate::Rapira for IndexMap<K, V, BuildHasherDefault<S>>
where
    K: Eq + core::hash::Hash,
    S: core::hash::Hasher + core::default::Default,
{
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

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> crate::Result<()>
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
    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;

        if len > VEC_MAX_CAP {
            return Err(crate::RapiraError::MaxCapacity);
        }

        let size = std::mem::size_of::<IndexMap<K, V, BuildHasherDefault<S>>>() * len;

        if size > VEC_MAX_SIZE_OF {
            return Err(crate::RapiraError::MaxSize);
        }

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
    fn from_slice_unchecked(slice: &mut &[u8]) -> crate::Result<Self>
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
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice_unsafe(slice)? as usize;
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
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> crate::Result<()> {
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
}

#[cfg(feature = "uuid")]
impl crate::Rapira for Uuid {
    const STATIC_SIZE: Option<usize> = Some(16);

    fn size(&self) -> usize {
        16
    }

    fn check_bytes(slice: &mut &[u8]) -> crate::Result<()> {
        <[u8; 16]>::check_bytes(slice)
    }

    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let bytes = uuid::Bytes::from_slice(slice)?;
        Ok(Self::from_bytes(bytes))
    }

    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let bytes = <[u8; 16]>::from_slice_unsafe(slice)?;
        Ok(Self::from_bytes(bytes))
    }

    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        let bytes = self.as_bytes();
        bytes.convert_to_bytes(slice, cursor);
    }
}
