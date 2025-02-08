use core::borrow::Borrow;

use super::ParseNodeIdError;

/// A unique string identifying a server for all time.
/// The maximum length of an id is 512 bytes.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeIdRef<'a, const N: usize = { u8::MAX as usize }>(&'a str);

impl<'a, const N: usize> NodeIdRef<'a, N> {
  /// The maximum length of an id is `N` bytes.
  pub const MAX_SIZE: usize = N;

  /// Creates a new `Id` from the source.
  pub fn new(src: &'a str) -> Result<Self, ParseNodeIdError> {
    if src.is_empty() {
      return Err(ParseNodeIdError::Empty);
    }

    if src.len() > Self::MAX_SIZE {
      return Err(ParseNodeIdError::too_large(Self::MAX_SIZE, src.len()));
    }

    Ok(Self(src))
  }

  /// converts the `Id` into a `&str`.
  #[inline]
  pub const fn as_str(&self) -> &'a str {
    self.0
  }

  /// Returns a byte slice.
  /// To convert the byte slice back into a string slice, use the [`core::str::from_utf8`] function.
  #[inline]
  pub const fn as_bytes(&self) -> &'a [u8] {
    self.0.as_bytes()
  }

  /// Converts to owned [`NodeId`](super::NodeId).
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
  #[inline]
  pub fn to_owned(&self) -> super::NodeId<N> {
    super::NodeId(self.0.into())
  }
}

impl<const N: usize> Borrow<str> for NodeIdRef<'_, N> {
  fn borrow(&self) -> &str {
    self.0
  }
}

impl<const N: usize> AsRef<str> for NodeIdRef<'_, N> {
  fn as_ref(&self) -> &str {
    self.0
  }
}

impl<const N: usize> core::fmt::Display for NodeIdRef<'_, N> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.0.fmt(f)
  }
}

impl<const N: usize> core::fmt::Debug for NodeIdRef<'_, N> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.0.fmt(f)
  }
}

impl<const N: usize> cheap_clone::CheapClone for NodeIdRef<'_, N> {}

impl<'a, const N: usize> TryFrom<&'a [u8]> for NodeIdRef<'a, N> {
  type Error = ParseNodeIdError;

  fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
    Self::new(core::str::from_utf8(value)?)
  }
}

impl<'a, const N: usize> TryFrom<&'a str> for NodeIdRef<'a, N> {
  type Error = ParseNodeIdError;

  fn try_from(value: &'a str) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

#[cfg(feature = "serde")]
const _: () = {
  use serde::{Deserialize, Deserializer, Serialize, Serializer};

  impl<const N: usize> Serialize for NodeIdRef<'_, N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      self.0.serialize(serializer)
    }
  }

  impl<'de, const N: usize> Deserialize<'de> for NodeIdRef<'de, N> {
    fn deserialize<D>(deserializer: D) -> Result<NodeIdRef<'de, N>, D::Error>
    where
      D: Deserializer<'de>,
    {
      let s = <&str>::deserialize(deserializer)?;
      NodeIdRef::new(s).map_err(serde::de::Error::custom)
    }
  }
};

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_basic() {
    let id = NodeIdRef::<16>::try_from(b"test".as_slice()).unwrap();
    assert_eq!(id.as_str(), "test");
    assert_eq!(id.as_ref(), "test");
    assert_eq!(id.as_bytes(), b"test");
    println!("{id}");
    println!("{id:?}");

    let _id = NodeIdRef::<16>::try_from("test1").unwrap();

    assert!(NodeIdRef::<20>::new("").is_err());
    assert!(NodeIdRef::<4>::new("aaaaa").is_err());
  }

  #[test]
  #[cfg(any(feature = "alloc", feature = "std"))]
  fn test_try_from() {
    let id = NodeIdRef::<16>::try_from("test").unwrap();
    assert_eq!(id.as_str(), "test");
    assert_eq!(id.as_ref(), "test");
    assert!(NodeIdRef::<16>::try_from("").is_err());
    assert!(NodeIdRef::<4>::try_from("aaaaa").is_err());

    let id = NodeIdRef::<16>::try_from("test".as_bytes()).unwrap();

    assert_eq!(id.as_str(), "test");
    assert_eq!(id.as_ref(), "test");
    assert!(NodeIdRef::<16>::try_from([].as_slice()).is_err());
    assert!(NodeIdRef::<4>::try_from("aaaaa").is_err());

    let id = id.to_owned();
    assert_eq!(id.as_str(), "test");
  }

  #[test]
  #[cfg(any(feature = "std", feature = "alloc"))]
  fn test_borrow() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    let id = NodeIdRef::<16>::try_from(b"test".as_slice()).unwrap();
    set.insert(id.clone());
    assert!(set.contains("test"));
  }

  #[cfg(feature = "serde")]
  #[test]
  fn test_serde() {
    let id = NodeIdRef::try_from("32").unwrap();
    let s = serde_json::to_string(&id).unwrap();
    let decoded: NodeIdRef = serde_json::from_str(&s).unwrap();
    assert_eq!(id, decoded);
  }
}
