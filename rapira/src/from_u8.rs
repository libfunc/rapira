pub trait FromU8: PartialEq<u8> + Sized {
    /// # Panics
    ///
    /// Panics if `u` is not equal to any variant
    fn from_u8(_: u8) -> Self;
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
pub struct EnumFromU8Error;

#[cfg(feature = "std")]
impl std::fmt::Display for EnumFromU8Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EnumFromU8Error")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for EnumFromU8Error {}
