#[cfg(feature = "smallvec")]
use smallvec::SmallVec;

pub trait MaxCapacity {
    const MAX_CAP: usize;
    const MAX_SIZE_OF: usize;
}

#[cfg(feature = "alloc")]
impl<T> MaxCapacity for Vec<T> {
    const MAX_CAP: usize = 512 * 1024;
    /// 2 gb
    const MAX_SIZE_OF: usize = 2 * 1024 * 1024 * 1024;
}

#[cfg(feature = "smallvec")]
impl<T, const CAP: usize> MaxCapacity for SmallVec<[T; CAP]> {
    const MAX_CAP: usize = 1024;

    /// 1gb
    const MAX_SIZE_OF: usize = 1024 * 1024 * 1024;
}
