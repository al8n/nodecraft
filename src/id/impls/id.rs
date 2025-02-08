use core::borrow::Borrow;

use smol_str03::SmolStr;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use std::{string::String, vec::Vec};

use super::ParseNodeIdError;

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
pub struct NodeId<const N: usize = { u8::MAX as usize }>(pub(crate) SmolStr);

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

#[cfg(any(feature = "alloc", feature = "std"))]
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

#[cfg(any(feature = "alloc", feature = "std"))]
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

#[cfg(feature = "arbitrary")]
const _: () = {
  use arbitrary::{Arbitrary, Unstructured};

  impl<'a, const N: usize> Arbitrary<'a> for NodeId<N> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
      // Generate a length between 1 and N
      let len = u.int_in_range(1..=N)?;

      // Generate random chars
      let mut id = String::with_capacity(len);
      while id.len() < len {
        let c = char::arbitrary(u)?;
        if id.len() + c.len_utf8() <= len {
          id.push(c);
        }
      }

      NodeId::try_from(id).map_err(|_| arbitrary::Error::IncorrectFormat)
    }
  }
};

#[cfg(feature = "quickcheck")]
const _: () = {
  use quickcheck::{Arbitrary, Gen};

  impl<const N: usize> Arbitrary for NodeId<N> {
    fn arbitrary(g: &mut Gen) -> Self {
      // Generate a length between 1 and N
      let len = usize::arbitrary(g) % N + 1;

      // Generate random chars
      let mut id = String::with_capacity(len);
      while id.len() < len {
        let c = char::arbitrary(g);
        if id.len() + c.len_utf8() <= len {
          id.push(c);
        }
      }

      NodeId::try_from(id).unwrap()
    }
  }
};

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
  #[cfg(any(feature = "alloc", feature = "std"))]
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
  #[cfg(any(feature = "std", feature = "alloc"))]
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

  #[cfg(feature = "serde")]
  #[quickcheck_macros::quickcheck]
  fn fuzzy_serde(node: NodeId) -> bool {
    let serialized = serde_json::to_string(&node).unwrap();
    let deserialized: NodeId = serde_json::from_str(&serialized).unwrap();
    node == deserialized
  }
}
