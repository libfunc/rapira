/// 512k
pub const VEC_MAX_CAP: usize = 512 * 1024;
/// 2 gb
pub const VEC_MAX_SIZE_OF: usize = 2 * 1024 * 1024 * 1024;

/// 512
#[cfg(feature = "smallvec")]
pub const SMALLVEC_MAX_CAP: usize = 512;
/// 1mb
#[cfg(feature = "smallvec")]
pub const SMALLVEC_MAX_SIZE_OF: usize = 1024 * 1024;
