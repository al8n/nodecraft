#[cfg(any(feature = "std", feature = "alloc"))]
mod id;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use id::*;

mod id_ref;
pub use id_ref::*;

/// Errors that can occur when transforming an [`NodeId`].
#[derive(Debug, thiserror::Error)]
pub enum ParseNodeIdError {
  /// Returned when the id is empty.
  #[error("id cannot be empty")]
  Empty,
  /// Returned when the id is too large.
  #[error("id is too large, maximum size is {maximum} bytes, but got {actual} bytes")]
  TooLarge {
    /// The maximum size of the [`NodeId`].
    maximum: usize,
    /// The actual size of the [`NodeId`].
    actual: usize,
  },
  /// Returned when the buffer is too small to encode the [`NodeId`].
  #[error("insufficient buffer, required: {required}, remaining: {remaining}")]
  InsufficientBuffer {
    /// The buffer size required to encode the [`NodeId`].
    required: u64,
    /// The buffer size remaining.
    remaining: u64,
  },
  /// Returned when the id is not a valid utf8 string.
  #[error(transparent)]
  Utf8Error(#[from] core::str::Utf8Error),
}

impl ParseNodeIdError {
  #[inline]
  const fn too_large(maximum: usize, actual: usize) -> Self {
    Self::TooLarge { maximum, actual }
  }
}
