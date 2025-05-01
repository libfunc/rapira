use crate::{LEN_SIZE, Rapira};

#[cfg(feature = "arrayvec")]
impl<T: crate::Rapira, const CAP: usize> crate::Rapira for arrayvec::ArrayVec<T, CAP> {
    const MIN_SIZE: usize = LEN_SIZE;

    #[inline]
    fn size(&self) -> usize {
        4 + match T::STATIC_SIZE {
            Some(size) => size * self.len(),
            None => self.iter().fold(0, |b, v| b + v.size()),
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
            return Err(crate::RapiraError::SliceLen);
        }
        let mut vec = Self::new_const();
        for _ in 0..len {
            let val = T::from_slice(slice)?;
            vec.push(val);
        }
        Ok(vec)
    }

    #[inline]
    unsafe fn from_slice_unchecked(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        unsafe {
            let len = u32::from_slice_unchecked(slice)? as usize;
            if len > CAP {
                return Err(crate::RapiraError::SliceLen);
            }
            let mut vec = Self::new_const();
            for _ in 0..len {
                let val = T::from_slice_unchecked(slice)?;
                vec.push_unchecked(val);
            }
            Ok(vec)
        }
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        unsafe {
            let len = usize::from_slice_unsafe(slice)?;
            let mut vec = Self::new_const();
            for _ in 0..len {
                let val = T::from_slice_unsafe(slice)?;
                vec.push_unchecked(val);
            }

            Ok(vec)
        }
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
impl<const CAP: usize> crate::Rapira for arrayvec::ArrayString<CAP> {
    const MIN_SIZE: usize = LEN_SIZE;

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
        crate::str_rapira::check_bytes::<()>(core::marker::PhantomData, slice)?;
        let size = len - slice.len();
        if size > CAP {
            Err(crate::RapiraError::SliceLen)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let s = crate::str_rapira::from_slice(slice)?;
        let s = Self::from(s).map_err(|_| crate::RapiraError::SliceLen)?;
        Ok(s)
    }

    #[inline]
    unsafe fn from_slice_unchecked(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let s = unsafe { crate::str_rapira::from_slice_unchecked(slice)? };
        let s = Self::from(s).map_err(|_| crate::RapiraError::SliceLen)?;
        Ok(s)
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let s = unsafe { crate::str_rapira::from_slice_unsafe(slice)? };
        let s = Self::from(s).map_err(|_| crate::RapiraError::SliceLen)?;
        Ok(s)
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        crate::str_rapira::convert_to_bytes(self, slice, cursor);
    }

    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> crate::Result<()> {
        crate::str_rapira::try_convert_to_bytes(self, slice, cursor)
    }
}

#[cfg(feature = "smallvec")]
impl<T: crate::Rapira, const CAP: usize> crate::Rapira for smallvec::SmallVec<[T; CAP]> {
    const MIN_SIZE: usize = LEN_SIZE;

    #[inline]
    fn size(&self) -> usize {
        4 + match T::STATIC_SIZE {
            Some(size) => size * self.len(),
            None => self.iter().fold(0, |b, v| b + v.size()),
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
        use crate::max_cap::{SMALLVEC_MAX_CAP, SMALLVEC_MAX_SIZE_OF};

        let len = u32::from_slice(slice)? as usize;

        if len > SMALLVEC_MAX_CAP {
            return Err(crate::RapiraError::MaxCapacity);
        }

        let size = std::mem::size_of::<Self>() * len;

        if size > SMALLVEC_MAX_SIZE_OF {
            return Err(crate::RapiraError::MaxSize);
        }

        let mut vec = Self::with_capacity(len);

        for _ in 0..len {
            let val = T::from_slice(slice)?;
            vec.push(val);
        }

        Ok(vec)
    }

    #[inline]
    unsafe fn from_slice_unchecked(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;
        let mut vec = Self::with_capacity(len);

        for _ in 0..len {
            let val = unsafe { T::from_slice_unchecked(slice)? };
            vec.push(val);
        }

        Ok(vec)
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        unsafe {
            let len = usize::from_slice_unsafe(slice)?;
            let mut vec = Self::with_capacity(len);

            for _ in 0..len {
                let val = T::from_slice_unsafe(slice)?;
                vec.push(val);
            }

            Ok(vec)
        }
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
impl crate::Rapira for bytes::Bytes {
    const MIN_SIZE: usize = LEN_SIZE;

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
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> crate::Result<()> {
        use crate::bytes_rapira;

        bytes_rapira::try_convert_to_bytes(self, slice, cursor)
    }

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use crate::bytes_rapira;

        let bytes = bytes_rapira::from_slice(slice)?;
        Ok(Self::copy_from_slice(bytes))
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use crate::bytes_rapira;

        unsafe {
            let bytes = bytes_rapira::from_slice_unsafe(slice)?;
            Ok(Self::copy_from_slice(bytes))
        }
    }
}

#[cfg(feature = "byteview")]
impl crate::Rapira for byteview::StrView {
    const MIN_SIZE: usize = LEN_SIZE;

    #[inline]
    fn size(&self) -> usize {
        use crate::str_rapira;

        str_rapira::size(self)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> crate::Result<()> {
        use crate::str_rapira;

        str_rapira::check_bytes::<()>(core::marker::PhantomData, slice)
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        use crate::str_rapira;

        str_rapira::convert_to_bytes(self, slice, cursor);
    }

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> crate::Result<()> {
        use crate::str_rapira;

        str_rapira::try_convert_to_bytes(self, slice, cursor)
    }

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use crate::str_rapira;

        let bytes = str_rapira::from_slice(slice)?;
        Ok(Self::from(bytes))
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use crate::str_rapira;

        unsafe {
            let bytes = str_rapira::from_slice_unsafe(slice)?;
            Ok(Self::from(bytes))
        }
    }
}

#[cfg(feature = "byteview")]
impl crate::Rapira for byteview::ByteView {
    const MIN_SIZE: usize = LEN_SIZE;

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
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> crate::Result<()> {
        use crate::bytes_rapira;

        bytes_rapira::try_convert_to_bytes(self, slice, cursor)
    }

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use crate::bytes_rapira;

        let bytes = bytes_rapira::from_slice(slice)?;
        Ok(Self::from(bytes))
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use crate::bytes_rapira;

        unsafe {
            let bytes = bytes_rapira::from_slice_unsafe(slice)?;
            Ok(Self::from(bytes))
        }
    }
}

#[cfg(feature = "fjall")]
impl crate::Rapira for fjall::Slice {
    const MIN_SIZE: usize = LEN_SIZE;

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
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> crate::Result<()> {
        use crate::bytes_rapira;

        bytes_rapira::try_convert_to_bytes(self, slice, cursor)
    }

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use crate::bytes_rapira;

        let bytes = bytes_rapira::from_slice(slice)?;
        Ok(Self::from(bytes))
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use crate::bytes_rapira;

        unsafe {
            let bytes = bytes_rapira::from_slice_unsafe(slice)?;
            Ok(Self::from(bytes))
        }
    }
}

#[cfg(feature = "inline-array")]
impl crate::Rapira for inline_array::InlineArray {
    const MIN_SIZE: usize = LEN_SIZE;

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
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> crate::Result<()> {
        use crate::bytes_rapira;

        bytes_rapira::try_convert_to_bytes(self, slice, cursor)
    }

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use crate::bytes_rapira;

        let bytes = bytes_rapira::from_slice(slice)?;
        Ok(Self::from(bytes))
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use crate::bytes_rapira;

        unsafe {
            let bytes = bytes_rapira::from_slice_unsafe(slice)?;
            Ok(Self::from(bytes))
        }
    }
}

#[cfg(feature = "zerocopy")]
pub mod zero {
    use core::{marker::PhantomData, mem::size_of};

    use zerocopy::{FromBytes, Immutable, IntoBytes};

    use crate::{extend, try_extend};

    pub const fn static_size<T>(_: PhantomData<T>) -> Option<usize>
    where
        T: Sized,
    {
        Some(size_of::<T>())
    }

    pub const fn min_size<T>(_: PhantomData<T>) -> usize
    where
        T: Sized,
    {
        size_of::<T>()
    }

    #[inline]
    pub fn size<T>(_: &T) -> usize
    where
        T: Sized,
    {
        size_of::<T>()
    }

    #[inline]
    pub fn check_bytes<T>(_: PhantomData<T>, slice: &mut &[u8]) -> crate::Result<()>
    where
        T: Sized,
    {
        let size = size_of::<T>();

        *slice = slice.get(size..).ok_or(crate::RapiraError::SliceLen)?;

        Ok(())
    }

    #[inline]
    pub fn from_slice<T>(slice: &mut &[u8]) -> crate::Result<T>
    where
        T: FromBytes + Sized,
    {
        let size = size_of::<T>();
        let bytes: &[u8] = slice.get(..size).ok_or(crate::RapiraError::SliceLen)?;

        *slice = slice.get(size..).ok_or(crate::RapiraError::SliceLen)?;

        let t: T = FromBytes::read_from_bytes(bytes)
            .map_err(|_| crate::RapiraError::Other("zerocopy error"))?;
        Ok(t)
    }

    #[inline]
    pub fn from_slice_unchecked<T>(slice: &mut &[u8]) -> crate::Result<T>
    where
        T: FromBytes + Sized,
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
        T: FromBytes + Sized,
    {
        unsafe {
            let size = size_of::<T>();
            let bytes: &[u8] = slice.get_unchecked(..size);
            *slice = slice.get_unchecked(size..);
            let t: T = FromBytes::read_from_bytes(bytes)
                .map_err(|_| crate::RapiraError::Other("zerocopy error"))?;
            Ok(t)
        }
    }

    #[inline]
    pub fn convert_to_bytes<T>(item: &T, slice: &mut [u8], cursor: &mut usize)
    where
        T: Immutable + IntoBytes + Sized,
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
        T: Immutable + IntoBytes + Sized,
    {
        let bytes = item.as_bytes();
        try_extend(slice, cursor, bytes)?;
        Ok(())
    }
}

#[cfg(feature = "serde_json")]
impl crate::Rapira for serde_json::Value {
    const MIN_SIZE: usize = 1;

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use serde_json::{Map, Number, Value};

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
                    let number = Number::from_f64(f).ok_or(crate::RapiraError::FloatIsNaN)?;
                    Ok(Value::Number(number))
                } else {
                    Err(crate::RapiraError::EnumVariant)
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
            _ => Err(crate::RapiraError::EnumVariant),
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
                    return Err(crate::RapiraError::EnumVariant);
                }
            }
            3 => {
                String::check_bytes(slice)?;
            }
            4 => {
                Vec::<Self>::check_bytes(slice)?;
            }
            5 => {
                let len = u32::from_slice(slice)? as usize;
                for _ in 0..len {
                    String::check_bytes(slice)?;
                    Self::check_bytes(slice)?;
                }
            }
            _ => return Err(crate::RapiraError::EnumVariant),
        }

        Ok(())
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use serde_json::{Map, Number};

        use crate::byte_rapira;

        unsafe {
            let byte = byte_rapira::from_slice_unsafe(slice)?;
            match byte {
                0 => Ok(Self::Null),
                1 => {
                    let b = bool::from_slice_unsafe(slice)?;
                    Ok(Self::Bool(b))
                }
                2 => {
                    let byte = byte_rapira::from_slice_unsafe(slice)?;
                    if byte == 0 {
                        let u = u64::from_slice_unsafe(slice)?;
                        Ok(Self::Number(u.into()))
                    } else if byte == 1 {
                        let i = i64::from_slice_unsafe(slice)?;
                        Ok(Self::Number(i.into()))
                    } else if byte == 2 {
                        let f = f64::from_slice_unsafe(slice)?;
                        let number = Number::from_f64(f).ok_or(crate::RapiraError::FloatIsNaN)?;
                        Ok(Self::Number(number))
                    } else {
                        Err(crate::RapiraError::EnumVariant)
                    }
                }
                3 => {
                    let s = String::from_slice_unsafe(slice)?;
                    Ok(Self::String(s))
                }
                4 => {
                    let vec = Vec::<Self>::from_slice_unsafe(slice)?;
                    Ok(Self::Array(vec))
                }
                5 => {
                    let len = usize::from_slice_unsafe(slice)?;
                    let mut map = Map::new();
                    for _ in 0..len {
                        let key = String::from_slice_unsafe(slice)?;
                        let val = Self::from_slice_unsafe(slice)?;
                        map.insert(key, val);
                    }
                    Ok(Self::Object(map))
                }
                _ => Err(crate::RapiraError::EnumVariant),
            }
        }
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        use crate::push;

        match self {
            Self::Null => {
                push(slice, cursor, 0);
            }
            Self::Bool(v) => {
                push(slice, cursor, 1);
                v.convert_to_bytes(slice, cursor);
            }
            Self::Number(n) => {
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
            Self::String(s) => {
                push(slice, cursor, 3);
                s.convert_to_bytes(slice, cursor);
            }
            Self::Array(a) => {
                push(slice, cursor, 4);
                a.convert_to_bytes(slice, cursor);
            }
            Self::Object(o) => {
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
            Self::Null => {
                try_push(slice, cursor, 0)?;
            }
            Self::Bool(v) => {
                try_push(slice, cursor, 1)?;
                v.convert_to_bytes(slice, cursor);
            }
            Self::Number(n) => {
                try_push(slice, cursor, 2)?;
                if let Some(u) = n.as_u64() {
                    try_push(slice, cursor, 0)?;
                    u.try_convert_to_bytes(slice, cursor)?;
                } else if let Some(i) = n.as_i64() {
                    try_push(slice, cursor, 1)?;
                    i.try_convert_to_bytes(slice, cursor)?;
                } else if let Some(f) = n.as_f64() {
                    try_push(slice, cursor, 2)?;
                    f.try_convert_to_bytes(slice, cursor)?;
                }
            }
            Self::String(s) => {
                try_push(slice, cursor, 3)?;
                s.try_convert_to_bytes(slice, cursor)?;
            }
            Self::Array(a) => {
                try_push(slice, cursor, 4)?;
                a.try_convert_to_bytes(slice, cursor)?;
            }
            Self::Object(o) => {
                try_push(slice, cursor, 5)?;
                let size: u32 = o.len() as u32;
                size.try_convert_to_bytes(slice, cursor)?;
                for (k, v) in o.iter() {
                    k.try_convert_to_bytes(slice, cursor)?;
                    v.try_convert_to_bytes(slice, cursor)?;
                }
            }
        }
        Ok(())
    }

    #[inline]
    fn size(&self) -> usize {
        1 + match self {
            Self::Null => 0,
            Self::Bool(_) => 1,
            Self::Number(_) => 1 + 8,
            Self::String(s) => s.size(),
            Self::Array(vec) => 4 + vec.iter().fold(0, |acc, item| acc + item.size()),
            Self::Object(v) => {
                4 + v
                    .iter()
                    .fold(0, |acc, item| acc + item.0.size() + item.1.size())
            }
        }
    }
}

#[cfg(feature = "rust_decimal")]
impl crate::Rapira for rust_decimal::Decimal {
    const STATIC_SIZE: Option<usize> = Some(16);
    const MIN_SIZE: usize = 16;

    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        unsafe {
            let bytes = <[u8; 16]>::from_slice_unsafe(slice)?;
            Ok(Self::deserialize(bytes))
        }
    }

    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let bytes = <[u8; 16]>::from_slice(slice)?;
        Ok(Self::deserialize(bytes))
    }

    fn check_bytes(_: &mut &[u8]) -> crate::Result<()> {
        Ok(())
    }

    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        self.serialize().convert_to_bytes(slice, cursor);
    }

    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> crate::Result<()> {
        self.serialize().try_convert_to_bytes(slice, cursor)
    }

    fn size(&self) -> usize {
        16
    }
}

#[cfg(feature = "compact_str")]
impl crate::Rapira for compact_str::CompactString {
    const MIN_SIZE: usize = LEN_SIZE;

    fn size(&self) -> usize {
        4 + self.len()
    }

    fn check_bytes(slice: &mut &[u8]) -> crate::Result<()> {
        crate::str_rapira::check_bytes::<()>(core::marker::PhantomData, slice)
    }

    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let s = crate::str_rapira::from_slice(slice)?;
        let s = Self::new(s);
        Ok(s)
    }

    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        unsafe {
            let s = crate::str_rapira::from_slice_unsafe(slice)?;
            let s = Self::new(s);
            Ok(s)
        }
    }

    unsafe fn from_slice_unchecked(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        unsafe {
            let s = crate::str_rapira::from_slice_unchecked(slice)?;
            let s = Self::new(s);
            Ok(s)
        }
    }

    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        crate::str_rapira::convert_to_bytes(self, slice, cursor);
    }

    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> crate::Result<()> {
        crate::str_rapira::try_convert_to_bytes(self, slice, cursor)
    }
}

#[cfg(feature = "ecow")]
impl crate::Rapira for ecow::EcoString {
    const MIN_SIZE: usize = LEN_SIZE;

    fn size(&self) -> usize {
        4 + self.len()
    }

    fn check_bytes(slice: &mut &[u8]) -> crate::Result<()> {
        crate::str_rapira::check_bytes::<()>(core::marker::PhantomData, slice)
    }

    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let s = crate::str_rapira::from_slice(slice)?;
        let s = Self::from(s);
        Ok(s)
    }

    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        unsafe {
            let s = crate::str_rapira::from_slice_unsafe(slice)?;
            let s = Self::from(s);
            Ok(s)
        }
    }

    unsafe fn from_slice_unchecked(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        unsafe {
            let s = crate::str_rapira::from_slice_unchecked(slice)?;
            let s = Self::from(s);
            Ok(s)
        }
    }

    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        crate::str_rapira::convert_to_bytes(self, slice, cursor);
    }

    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> crate::Result<()> {
        crate::str_rapira::try_convert_to_bytes(self, slice, cursor)
    }
}

#[cfg(feature = "ecow")]
impl<T> crate::Rapira for ecow::EcoVec<T>
where
    T: crate::Rapira + Clone,
{
    const MIN_SIZE: usize = LEN_SIZE;

    #[inline]
    fn size(&self) -> usize {
        4 + match T::STATIC_SIZE {
            Some(size) => size * self.len(),
            None => self.iter().fold(0, |b, v| b + v.size()),
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
        use crate::max_cap::{SMALLVEC_MAX_CAP, SMALLVEC_MAX_SIZE_OF};

        let len = u32::from_slice(slice)? as usize;

        if len > SMALLVEC_MAX_CAP {
            return Err(crate::RapiraError::MaxCapacity);
        }

        let size = std::mem::size_of::<Self>() * len;

        if size > SMALLVEC_MAX_SIZE_OF {
            return Err(crate::RapiraError::MaxSize);
        }

        let mut vec = Self::with_capacity(len);

        for _ in 0..len {
            let val = T::from_slice(slice)?;
            vec.push(val);
        }

        Ok(vec)
    }

    #[inline]
    unsafe fn from_slice_unchecked(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;
        let mut vec = Self::with_capacity(len);

        for _ in 0..len {
            let val = unsafe { T::from_slice_unchecked(slice)? };
            vec.push(val);
        }

        Ok(vec)
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        unsafe {
            let len = usize::from_slice_unsafe(slice)?;
            let mut vec = Self::with_capacity(len);

            let iter = core::iter::repeat_with(|| T::from_slice_unsafe(slice)).take(len);

            for item in iter {
                let val = item?;
                vec.push(val);
            }

            Ok(vec)
        }
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

#[cfg(feature = "indexmap")]
impl<K: crate::Rapira, V: crate::Rapira, S> crate::Rapira
    for indexmap::IndexMap<K, V, core::hash::BuildHasherDefault<S>>
where
    K: Eq + core::hash::Hash,
    S: core::hash::Hasher + core::default::Default,
{
    const MIN_SIZE: usize = LEN_SIZE;

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
        use crate::max_cap::{VEC_MAX_CAP, VEC_MAX_SIZE_OF};

        let len = u32::from_slice(slice)? as usize;

        if len > VEC_MAX_CAP {
            return Err(crate::RapiraError::MaxCapacity);
        }

        let size = std::mem::size_of::<Self>() * len;

        if size > VEC_MAX_SIZE_OF {
            return Err(crate::RapiraError::MaxSize);
        }

        let mut map = Self::with_capacity_and_hasher(len, Default::default());
        for _ in 0..len {
            let key = K::from_slice(slice)?;
            let value = V::from_slice(slice)?;
            map.insert(key, value);
        }
        Ok(map)
    }

    #[inline]
    unsafe fn from_slice_unchecked(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;
        let mut map = Self::with_capacity_and_hasher(len, Default::default());
        for _ in 0..len {
            unsafe {
                let key = K::from_slice_unchecked(slice)?;
                let value = V::from_slice_unchecked(slice)?;
                map.insert(key, value);
            }
        }
        Ok(map)
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        unsafe {
            let len = u32::from_slice_unsafe(slice)? as usize;
            let mut map = Self::with_capacity_and_hasher(len, Default::default());
            for _ in 0..len {
                let key = K::from_slice_unsafe(slice)?;
                let value = V::from_slice_unsafe(slice)?;
                map.insert(key, value);
            }
            Ok(map)
        }
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
impl crate::Rapira for uuid::Uuid {
    const STATIC_SIZE: Option<usize> = Some(16);
    const MIN_SIZE: usize = 16;

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
        unsafe {
            let bytes = <[u8; 16]>::from_slice_unsafe(slice)?;
            Ok(Self::from_bytes(bytes))
        }
    }

    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        let bytes = self.as_bytes();
        bytes.convert_to_bytes(slice, cursor);
    }

    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> crate::Result<()> {
        let bytes = self.as_bytes();
        bytes.try_convert_to_bytes(slice, cursor)
    }
}

#[cfg(feature = "time")]
impl Rapira for time::Date {
    const STATIC_SIZE: Option<usize> = Some(4);
    const MIN_SIZE: usize = 4;

    fn size(&self) -> usize {
        4
    }

    fn check_bytes(slice: &mut &[u8]) -> crate::Result<()> {
        let val = i32::from_slice(slice)?;
        Self::from_julian_day(val).map_err(|_| crate::RapiraError::Datetime)?;
        Ok(())
    }

    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let julian_day = i32::from_slice(slice)?;
        Self::from_julian_day(julian_day).map_err(|_| crate::RapiraError::Datetime)
    }

    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        self.to_julian_day().convert_to_bytes(slice, cursor);
    }
}

#[cfg(feature = "solana")]
impl Rapira for solana_pubkey::Pubkey {
    const STATIC_SIZE: Option<usize> = Some(32);
    const MIN_SIZE: usize = 32;

    fn size(&self) -> usize {
        32
    }

    fn check_bytes(slice: &mut &[u8]) -> crate::Result<()> {
        <[u8; 32]>::check_bytes(slice)
    }

    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let bytes = <[u8; 32]>::from_slice(slice)?;
        Ok(Self::from(bytes))
    }

    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        self.as_array().convert_to_bytes(slice, cursor);
    }
}

#[cfg(feature = "solana")]
impl Rapira for solana_signature::Signature {
    const STATIC_SIZE: Option<usize> = Some(64);
    const MIN_SIZE: usize = 64;

    fn size(&self) -> usize {
        64
    }

    fn check_bytes(slice: &mut &[u8]) -> crate::Result<()> {
        <[u8; 64]>::check_bytes(slice)
    }

    fn from_slice(slice: &mut &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let bytes = <[u8; 64]>::from_slice(slice)?;
        Ok(Self::from(bytes))
    }

    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        let end = *cursor + 64;
        slice[*cursor..end].copy_from_slice(self.as_ref());
        *cursor = end;
    }
}
