#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::string::String;

#[cfg(feature = "std")]
use thiserror::Error;

#[cfg_attr(feature = "std", derive(Error, Debug))]
pub enum RapiraError {
    #[cfg_attr(feature = "std", error("iter next error"))]
    IterNextError,
    #[cfg_attr(feature = "std", error("string from utf8 error"))]
    StringTypeError,
    #[cfg_attr(feature = "std", error("datetime error"))]
    DatetimeError,
    #[cfg_attr(feature = "std", error("map insert error: args next error"))]
    MapInsertError,
    #[cfg_attr(feature = "std", error("enum variant error"))]
    EnumVariantError,
    #[cfg_attr(feature = "std", error("non zero to zero"))]
    FloatIsNaNError,
    #[cfg_attr(feature = "std", error("float is NaN"))]
    DecimalError,
    #[cfg_attr(feature = "std", error("decimal scale error"))]
    NonZeroError,
    #[cfg_attr(feature = "std", error("slice len error"))]
    SliceLenError,
    #[cfg_attr(feature = "std", error("from arr not implemented"))]
    FromArrNotImplemented,
    #[cfg(feature = "std")]
    #[cfg_attr(feature = "std", error("other error: {0}"))]
    OtherError(String),
}

pub type Result<T, E = RapiraError> = core::result::Result<T, E>;
