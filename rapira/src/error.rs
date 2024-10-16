use core::array::TryFromSliceError;

#[cfg(feature = "std")]
use thiserror::Error;

#[cfg_attr(feature = "std", derive(Error, Debug))]
pub enum RapiraError {
    #[cfg_attr(feature = "std", error("iter next error"))]
    IterNext,
    #[cfg_attr(feature = "std", error("string from utf8 error"))]
    StringType,
    #[cfg_attr(feature = "std", error("datetime error"))]
    Datetime,
    #[cfg_attr(feature = "std", error("map insert error: args next error"))]
    MapInsert,
    #[cfg_attr(feature = "std", error("enum variant error"))]
    EnumVariant,
    #[cfg_attr(feature = "std", error("non zero to zero"))]
    FloatIsNaN,
    #[cfg_attr(feature = "std", error("float is NaN"))]
    Decimal,
    #[cfg_attr(feature = "std", error("decimal scale error"))]
    NonZero,
    #[cfg_attr(feature = "std", error("slice len error"))]
    SliceLen,
    #[cfg_attr(feature = "std", error("from arr not implemented"))]
    FromArrNotImplemented,
    #[cfg_attr(feature = "std", error("max size error"))]
    MaxSize,
    #[cfg_attr(feature = "std", error("max capacity error"))]
    MaxCapacity,
    #[cfg_attr(feature = "std", error(transparent))]
    TryFromSlice(#[cfg_attr(feature = "std", from)] TryFromSliceError),
    #[cfg(feature = "std")]
    #[cfg_attr(feature = "std", error("other error: {0}"))]
    Other(&'static str),
}

pub type Result<T, E = RapiraError> = core::result::Result<T, E>;
