use crate::{
    byte_rapira,
    max_cap::{VEC_MAX_CAP, VEC_MAX_SIZE_OF},
    primitive::bytes_rapira,
    push, str_rapira, Rapira, RapiraError, Result,
};
use alloc::borrow::Cow;

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, collections::BTreeMap, string::String, vec::Vec};
#[cfg(feature = "std")]
use std::net::{IpAddr, Ipv6Addr, SocketAddrV6};

#[cfg(feature = "alloc")]
impl Rapira for String {
    #[inline]
    fn size(&self) -> usize {
        str_rapira::size(self)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        str_rapira::check_bytes::<()>(core::marker::PhantomData, slice)
    }

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let s = str_rapira::from_slice(slice)?;
        let s = s.to_owned();
        Ok(s)
    }

    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let s = str_rapira::from_slice_unchecked(slice)?;
        Ok(s.to_owned())
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let s = str_rapira::from_slice_unsafe(slice)?;
        Ok(s.to_owned())
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        str_rapira::convert_to_bytes(self, slice, cursor);
    }

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        str_rapira::try_convert_to_bytes(self, slice, cursor)
    }
}

#[cfg(feature = "alloc")]
impl Rapira for Vec<u8> {
    #[inline]
    fn size(&self) -> usize {
        bytes_rapira::size(self)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        bytes_rapira::check_bytes::<()>(core::marker::PhantomData, slice)
    }

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let bytes = bytes_rapira::from_slice(slice)?;
        Ok(bytes.to_vec())
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let bytes = bytes_rapira::from_slice_unsafe(slice)?;
        Ok(bytes.to_vec())
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        bytes_rapira::convert_to_bytes(self, slice, cursor);
    }

    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        bytes_rapira::try_convert_to_bytes(self, slice, cursor)
    }
}

#[cfg(feature = "alloc")]
impl<T: Rapira> Rapira for Vec<T> {
    #[inline]
    fn size(&self) -> usize {
        match T::STATIC_SIZE {
            Some(size) => 4 + (size * self.len()),
            None => 4 + self.iter().fold(0, |b, v| b + v.size()),
        }
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
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let len = u32::from_slice(slice)? as usize;

        if len > VEC_MAX_CAP {
            return Err(RapiraError::MaxCapacity);
        }

        let size = std::mem::size_of::<Vec<T>>() * len;

        if size > VEC_MAX_SIZE_OF {
            return Err(RapiraError::MaxSize);
        }

        let mut vec: Vec<T> = Vec::with_capacity(len);

        for _ in 0..len {
            let val = T::from_slice(slice)?;
            vec.push(val);
        }

        Ok(vec)
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
        let len = u32::from_slice_unsafe(slice)? as usize;
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
        let len = u32::from_slice_unsafe(slice)?;
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
}

#[cfg(feature = "alloc")]
impl<'a> Rapira for Cow<'a, str> {
    #[inline]
    fn size(&self) -> usize {
        str_rapira::size(self)
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        str_rapira::check_bytes::<()>(core::marker::PhantomData, slice)
    }

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let s = str_rapira::from_slice(slice)?;
        let s = Cow::Owned(s.to_owned());
        Ok(s)
    }

    #[inline]
    fn from_slice_unchecked(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let s = str_rapira::from_slice_unchecked(slice)?;
        let s = Cow::Owned(s.to_owned());
        Ok(s)
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let s = str_rapira::from_slice_unsafe(slice)?;
        let s = Cow::Owned(s.to_owned());
        Ok(s)
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        str_rapira::convert_to_bytes(self, slice, cursor);
    }

    #[inline]
    fn try_convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) -> Result<()> {
        str_rapira::try_convert_to_bytes(self, slice, cursor)
    }
}

#[cfg(feature = "std")]
impl Rapira for IpAddr {
    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let b = byte_rapira::from_slice(slice)?;
        if b == 0 {
            let v4 = <[u8; 4]>::from_slice(slice)?;
            Ok(IpAddr::from(v4))
        } else {
            let v6 = Ipv6Addr::from_slice(slice)?;
            Ok(IpAddr::from(v6))
        }
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        let b = byte_rapira::from_slice(slice)?;

        if b == 0 {
            <[u8; 4]>::check_bytes(slice)?;
        } else {
            Ipv6Addr::check_bytes(slice)?;
        }

        Ok(())
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let b = byte_rapira::from_slice_unsafe(slice)?;
        if b == 0 {
            let v4 = <[u8; 4]>::from_slice_unsafe(slice)?;
            Ok(IpAddr::from(v4))
        } else {
            let v6 = Ipv6Addr::from_slice_unsafe(slice)?;
            Ok(IpAddr::from(v6))
        }
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        match self {
            IpAddr::V4(v4) => {
                push(slice, cursor, 0);
                v4.octets().convert_to_bytes(slice, cursor);
            }
            IpAddr::V6(v6) => {
                push(slice, cursor, 1);
                v6.convert_to_bytes(slice, cursor);
            }
        }
    }

    #[inline]
    fn size(&self) -> usize {
        1 + match self {
            IpAddr::V4(_) => 4,
            IpAddr::V6(_) => 16,
        }
    }
}

#[cfg(feature = "std")]
impl Rapira for Ipv6Addr {
    const STATIC_SIZE: Option<usize> = Some(16);

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let v6 = <[u8; 16]>::from_slice(slice)?;
        Ok(Ipv6Addr::from(v6))
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        <[u8; 16]>::check_bytes(slice)?;

        Ok(())
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let v6 = <[u8; 16]>::from_slice_unsafe(slice)?;
        Ok(Ipv6Addr::from(v6))
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        self.octets().convert_to_bytes(slice, cursor);
    }

    #[inline]
    fn size(&self) -> usize {
        16
    }
}

#[cfg(feature = "std")]
impl Rapira for SocketAddrV6 {
    const STATIC_SIZE: Option<usize> = Some(16 + 2);

    #[inline]
    fn from_slice(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let ip = Ipv6Addr::from_slice(slice)?;
        let port = u16::from_slice(slice)?;
        Ok(SocketAddrV6::new(ip, port, 0, 0))
    }

    #[inline]
    fn check_bytes(slice: &mut &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        Ipv6Addr::check_bytes(slice)?;
        u16::check_bytes(slice)?;

        Ok(())
    }

    #[inline]
    unsafe fn from_slice_unsafe(slice: &mut &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let ip = Ipv6Addr::from_slice_unsafe(slice)?;
        let port = u16::from_slice_unsafe(slice)?;
        Ok(SocketAddrV6::new(ip, port, 0, 0))
    }

    #[inline]
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        self.ip().convert_to_bytes(slice, cursor);
        self.port().convert_to_bytes(slice, cursor);
    }

    #[inline]
    fn size(&self) -> usize {
        16 + 2
    }
}
