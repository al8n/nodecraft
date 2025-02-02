use core::borrow::Borrow;

use smol_str03::SmolStr;

// use crate::Id;

#[cfg(not(all(feature = "std", feature = "alloc")))]
use ::alloc::{string::String, vec::Vec};

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

/// A unique string identifying a server for all time.
/// The maximum length of an id is 512 bytes.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(
  feature = "rkyv",
  derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(
  feature = "rkyv",
  rkyv(compare(PartialEq), derive(PartialEq, Eq, PartialOrd, Ord, Hash),)
)]
pub struct NodeId<const N: usize = { u8::MAX as usize }>(SmolStr);

// impl<const N: usize> Id for NodeId<N> {}

impl<const N: usize> NodeId<N> {
  /// The maximum length of an id is `N` bytes.
  pub const MAX_SIZE: usize = N;

  /// Creates a new `Id` from the source.
  pub fn new<T: AsRef<str>>(src: T) -> Result<Self, ParseNodeIdError> {
    let src = src.as_ref();
    if src.is_empty() {
      return Err(ParseNodeIdError::Empty);
    }

    if src.len() > Self::MAX_SIZE {
      return Err(ParseNodeIdError::too_large(Self::MAX_SIZE, src.len()));
    }

    Ok(Self(SmolStr::new(src)))
  }

  /// converts the `Id` into a `&str`.
  pub fn as_str(&self) -> &str {
    self.0.as_ref()
  }

  /// Returns a byte slice.
  /// To convert the byte slice back into a string slice, use the [`core::str::from_utf8`] function.
  pub fn as_bytes(&self) -> &[u8] {
    self.0.as_bytes()
  }
}

impl<const N: usize> core::str::FromStr for NodeId<N> {
  type Err = ParseNodeIdError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::new(s)
  }
}

impl<const N: usize> Borrow<str> for NodeId<N> {
  fn borrow(&self) -> &str {
    self.as_str()
  }
}

impl<const N: usize> AsRef<str> for NodeId<N> {
  fn as_ref(&self) -> &str {
    self.0.as_ref()
  }
}

impl<const N: usize> core::fmt::Display for NodeId<N> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.0.fmt(f)
  }
}

impl<const N: usize> core::fmt::Debug for NodeId<N> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.0.fmt(f)
  }
}

impl<const N: usize> cheap_clone::CheapClone for NodeId<N> {}

impl<const N: usize> From<NodeId<N>> for SmolStr {
  fn from(id: NodeId<N>) -> Self {
    id.0
  }
}

impl<const N: usize> TryFrom<&[u8]> for NodeId<N> {
  type Error = ParseNodeIdError;

  fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
    Self::new(core::str::from_utf8(value)?)
  }
}

#[cfg(feature = "alloc")]
impl<const N: usize> TryFrom<Vec<u8>> for NodeId<N> {
  type Error = ParseNodeIdError;

  fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
    let s = String::from_utf8(value).map_err(|e| ParseNodeIdError::Utf8Error(e.utf8_error()))?;

    if s.len() > Self::MAX_SIZE {
      return Err(ParseNodeIdError::too_large(Self::MAX_SIZE, s.len()));
    }

    if s.is_empty() {
      return Err(ParseNodeIdError::Empty);
    }

    Ok(Self(s.into()))
  }
}

#[cfg(feature = "alloc")]
impl<const N: usize> TryFrom<String> for NodeId<N> {
  type Error = ParseNodeIdError;

  fn try_from(s: String) -> Result<Self, Self::Error> {
    if s.len() > Self::MAX_SIZE {
      return Err(ParseNodeIdError::too_large(Self::MAX_SIZE, s.len()));
    }

    if s.is_empty() {
      return Err(ParseNodeIdError::Empty);
    }

    Ok(Self(s.into()))
  }
}

#[cfg(test)]
mod tests {
  use core::str::FromStr;

  use rand::{distr::Alphanumeric, rng};

  use super::*;

  impl NodeId {
    fn random(size: usize) -> Self {
      use rand::Rng;
      let id = rng()
        .sample_iter(Alphanumeric)
        .take(size)
        .collect::<Vec<u8>>();

      NodeId::try_from(id).unwrap()
    }
  }

  #[test]
  fn test_basic() {
    let id = NodeId::<16>::try_from(b"test".as_slice()).unwrap();
    assert_eq!(id.as_str(), "test");
    assert_eq!(id.as_ref(), "test");
    assert_eq!(id.as_bytes(), b"test");
    println!("{id}");
    println!("{id:?}");

    let _id = NodeId::<16>::from_str("test1").unwrap();

    assert!(NodeId::<20>::new("").is_err());
    assert!(NodeId::<512>::new("a".repeat(513)).is_err());
  }

  #[test]
  #[cfg(feature = "alloc")]
  fn test_try_from() {
    let id = NodeId::<16>::try_from(String::from("test")).unwrap();
    assert_eq!(id.as_str(), "test");
    assert_eq!(id.as_ref(), "test");
    assert!(NodeId::<16>::try_from(String::new()).is_err());
    assert!(NodeId::<512>::try_from("a".repeat(513)).is_err());

    let id = NodeId::<16>::try_from(Vec::from("test".as_bytes())).unwrap();

    assert_eq!(id.as_str(), "test");
    assert_eq!(id.as_ref(), "test");
    assert!(NodeId::<16>::try_from(Vec::new()).is_err());
    assert!(NodeId::<512>::try_from("a".repeat(513).into_bytes()).is_err());

    let id = SmolStr::from(id);
    assert_eq!(id.as_str(), "test");
  }

  #[test]
  #[cfg(feature = "std")]
  fn test_borrow() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    let id = NodeId::<16>::try_from(b"test".as_slice()).unwrap();
    set.insert(id.clone());
    assert!(set.contains("test"));
  }

  #[cfg(feature = "serde")]
  #[test]
  fn test_serde() {
    let id = NodeId::random(32);
    let s = serde_json::to_string(&id).unwrap();
    let decoded: NodeId = serde_json::from_str(&s).unwrap();
    assert_eq!(id, decoded);
  }
}
