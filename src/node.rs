use core::fmt::Display;

use cheap_clone::CheapClone;

/// Node is consist of id and address, which can be used as a identifier in a distributed system.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
  feature = "rkyv",
  derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "rkyv", archive(check_bytes, compare(PartialEq)))]
pub struct Node<I, A> {
  id: I,
  address: A,
}

impl<I: Display, A: Display> Display for Node<I, A> {
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}({})", self.id, self.address)
  }
}

impl<I, A> From<(I, A)> for Node<I, A> {
  #[inline]
  fn from((id, address): (I, A)) -> Self {
    Self { id, address }
  }
}

impl<I, A> Node<I, A> {
  /// Create a new node with id and address.
  #[inline]
  pub const fn new(id: I, address: A) -> Self {
    Self { id, address }
  }

  /// Returns the id of the node.
  #[inline]
  pub const fn id(&self) -> &I {
    &self.id
  }

  /// Returns the address of the node.
  #[inline]
  pub const fn address(&self) -> &A {
    &self.address
  }

  /// Set the address of the node.
  #[inline]
  pub fn set_address(&mut self, address: A) -> &mut Self {
    self.address = address;
    self
  }

  /// Set the id of the node.
  #[inline]
  pub fn set_id(&mut self, id: I) -> &mut Self {
    self.id = id;
    self
  }

  /// Set the address of the node. (Builder pattern)
  #[inline]
  pub fn with_address(mut self, address: A) -> Self {
    self.address = address;
    self
  }

  /// Set the id of the node. (Builder pattern)
  #[inline]
  pub fn with_id(mut self, id: I) -> Self {
    self.id = id;
    self
  }

  /// Consumes the node and returns the id and address of the node.
  #[inline]
  pub fn into_components(self) -> (I, A) {
    (self.id, self.address)
  }

  /// Maps an `Node<I, A>` to `Node<I, U>` by applying a function to the current node.
  ///
  /// # Example
  ///
  /// ```
  /// use nodecraft::Node;
  ///
  /// let node = Node::new("test", 100u64);
  /// let node = node.map_address(|address| address.to_string());
  /// assert_eq!(node.address(), "100");
  #[inline]
  pub fn map_address<U>(self, f: impl FnOnce(A) -> U) -> Node<I, U> {
    Node {
      id: self.id,
      address: f(self.address),
    }
  }

  /// Maps an `Node<I, A>` to `Node<U, A>` by applying a function to the current node.
  ///
  /// # Example
  ///
  /// ```
  /// use nodecraft::Node;
  ///
  /// let node = Node::new(1u64, 100u64);
  /// let node = node.map_id(|id| id.to_string());
  /// assert_eq!(node.id(), "1");
  /// ```
  #[inline]
  pub fn map_id<U>(self, f: impl FnOnce(I) -> U) -> Node<U, A> {
    Node {
      id: f(self.id),
      address: self.address,
    }
  }

  /// Maps an `Node<I, A>` to `Node<U, V>` by applying a function to the current node.
  ///
  /// # Example
  ///
  /// ```
  /// use nodecraft::Node;
  ///
  /// let node = Node::new(1u64, 100u64);
  ///
  /// let node = node.map(|id, address| (id.to_string(), address.to_string()));
  ///
  /// assert_eq!(node.id(), "1");
  /// assert_eq!(node.address(), "100");
  /// ```
  #[inline]
  pub fn map<U, V>(self, f: impl FnOnce(I, A) -> (U, V)) -> Node<U, V> {
    let (id, address) = f(self.id, self.address);
    Node { id, address }
  }
}

impl<I: CheapClone, A: CheapClone> CheapClone for Node<I, A> {
  #[inline]
  fn cheap_clone(&self) -> Self {
    Self {
      id: self.id.cheap_clone(),
      address: self.address.cheap_clone(),
    }
  }
}

/// Error type returned when transforming a `Node`.
#[cfg(feature = "transformable")]
pub enum NodeTransformError<I: transformable::Transformable, A: transformable::Transformable> {
  /// Error occurred when transforming the id.
  Id(I::Error),
  /// Error occurred when transforming the address.
  Address(A::Error),
  /// The encode buffer is too small.
  EncodeBufferTooSmall,
  /// The data is truncated or corrupted.
  Corrupted,
}

#[cfg(feature = "transformable")]
const _: () = {
  use byteorder::{ByteOrder, NetworkEndian};
  use transformable::Transformable;

  impl<I: Transformable, A: Transformable> Clone for NodeTransformError<I, A>
  where
    I::Error: Clone,
    A::Error: Clone,
  {
    #[inline]
    fn clone(&self) -> Self {
      match self {
        Self::Id(e) => Self::Id(e.clone()),
        Self::Address(e) => Self::Address(e.clone()),
        Self::EncodeBufferTooSmall => Self::EncodeBufferTooSmall,
        Self::Corrupted => Self::Corrupted,
      }
    }
  }

  impl<I: Transformable, A: Transformable> Copy for NodeTransformError<I, A>
  where
    I::Error: Copy,
    A::Error: Copy,
  {
  }

  impl<I: Transformable, A: Transformable> PartialEq for NodeTransformError<I, A>
  where
    I::Error: PartialEq,
    A::Error: PartialEq,
  {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
      match (self, other) {
        (Self::Id(e1), Self::Id(e2)) => e1 == e2,
        (Self::Address(e1), Self::Address(e2)) => e1 == e2,
        (Self::EncodeBufferTooSmall, Self::EncodeBufferTooSmall) => true,
        (Self::Corrupted, Self::Corrupted) => true,
        _ => false,
      }
    }
  }

  impl<I: Transformable, A: Transformable> Eq for NodeTransformError<I, A>
  where
    I::Error: Eq,
    A::Error: Eq,
  {
  }

  impl<I: Transformable, A: Transformable> core::fmt::Debug for NodeTransformError<I, A>
  where
    I::Error: core::fmt::Debug,
    A::Error: core::fmt::Debug,
  {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        Self::Id(e) => write!(f, "Id({:?})", e),
        Self::Address(e) => write!(f, "Address({:?})", e),
        Self::EncodeBufferTooSmall => write!(f, "EncodeBufferTooSmall"),
        Self::Corrupted => write!(f, "Corrupted"),
      }
    }
  }

  impl<I: Transformable, A: Transformable> core::fmt::Display for NodeTransformError<I, A>
  where
    I::Error: core::fmt::Display,
    A::Error: core::fmt::Display,
  {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        Self::Id(e) => write!(f, "{e}"),
        Self::Address(e) => write!(f, "{e}"),
        Self::EncodeBufferTooSmall => write!(f, "encode buffer too small"),
        Self::Corrupted => write!(f, "the data is truncated or corrupted"),
      }
    }
  }

  #[cfg(feature = "std")]
  impl<I: Transformable, A: Transformable> std::error::Error for NodeTransformError<I, A>
  where
    I::Error: std::error::Error,
    A::Error: std::error::Error,
  {
  }

  const MAX_NODE_LEN: usize = core::mem::size_of::<u32>();

  impl<I: Transformable, A: Transformable> Transformable for Node<I, A>
  where
    I: Transformable,
    A: Transformable,
  {
    type Error = NodeTransformError<I, A>;

    fn encode(&self, dst: &mut [u8]) -> Result<usize, Self::Error> {
      let mut offset = 0;
      let encoded_len = self.encoded_len();
      if dst.len() < encoded_len {
        return Err(NodeTransformError::EncodeBufferTooSmall);
      }
      NetworkEndian::write_u32(&mut dst[offset..MAX_NODE_LEN], encoded_len as u32);
      offset += MAX_NODE_LEN;
      offset += self
        .id
        .encode(&mut dst[offset..])
        .map_err(Self::Error::Id)?;
      offset += self
        .address
        .encode(&mut dst[offset..])
        .map_err(Self::Error::Address)?;
      debug_assert_eq!(
        offset, encoded_len,
        "expected encoded_len: {}, actual: {}",
        encoded_len, offset
      );
      Ok(offset)
    }

    fn encoded_len(&self) -> usize {
      MAX_NODE_LEN + self.id.encoded_len() + self.address.encoded_len()
    }

    fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error>
    where
      Self: Sized,
    {
      let mut offset = 0;
      if src.len() < MAX_NODE_LEN {
        return Err(NodeTransformError::Corrupted);
      }

      let encoded_len = NetworkEndian::read_u32(&src[offset..MAX_NODE_LEN]) as usize;
      offset += MAX_NODE_LEN;

      if src.len() < encoded_len {
        return Err(NodeTransformError::Corrupted);
      }

      let (id_len, id) = I::decode(&src[offset..]).map_err(Self::Error::Id)?;
      offset += id_len;

      let (address_len, address) = A::decode(&src[offset..]).map_err(Self::Error::Address)?;
      offset += address_len;

      debug_assert_eq!(
        offset, encoded_len,
        "expected read {} bytes, actual read {} bytes",
        encoded_len, offset
      );

      Ok((offset, Self { id, address }))
    }
  }
};

#[cfg(feature = "rkyv")]
const _: () = {
  use rkyv::Archive;

  impl<I: Archive, A: Archive> Clone for ArchivedNode<I, A>
  where
    I::Archived: Clone,
    A::Archived: Clone,
  {
    #[inline]
    fn clone(&self) -> Self {
      Self {
        id: self.id.clone(),
        address: self.address.clone(),
      }
    }
  }

  impl<I: Archive, A: Archive> CheapClone for ArchivedNode<I, A>
  where
    I::Archived: CheapClone,
    A::Archived: CheapClone,
  {
    #[inline]
    fn cheap_clone(&self) -> Self {
      Self {
        id: self.id.cheap_clone(),
        address: self.address.cheap_clone(),
      }
    }
  }

  impl<I: Archive, A: Archive> Copy for ArchivedNode<I, A>
  where
    I::Archived: Copy,
    A::Archived: Copy,
  {
  }

  impl<I: Archive, A: Archive> PartialEq for ArchivedNode<I, A>
  where
    I::Archived: PartialEq,
    A::Archived: PartialEq,
  {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
      self.id == other.id && self.address == other.address
    }
  }

  impl<I: Archive, A: Archive> Eq for ArchivedNode<I, A>
  where
    I::Archived: Eq,
    A::Archived: Eq,
  {
  }

  impl<I: Archive, A: Archive> core::hash::Hash for ArchivedNode<I, A>
  where
    I::Archived: core::hash::Hash,
    A::Archived: core::hash::Hash,
  {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
      self.id.hash(state);
      self.address.hash(state);
    }
  }

  impl<I: Archive, A: Archive> core::fmt::Debug for ArchivedNode<I, A>
  where
    I::Archived: core::fmt::Debug,
    A::Archived: core::fmt::Debug,
  {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("ArchivedNode")
        .field("id", &self.id)
        .field("address", &self.address)
        .finish()
    }
  }

  impl<I: Archive, A: Archive> core::fmt::Display for ArchivedNode<I, A>
  where
    I::Archived: core::fmt::Display,
    A::Archived: core::fmt::Display,
  {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      write!(f, "{}({})", self.id, self.address)
    }
  }
};

#[cfg(test)]
mod tests {
  use super::*;
  use rand::distributions::Alphanumeric;
  use smol_str::SmolStr;

  fn random(size: usize) -> Node<SmolStr, u64> {
    use rand::{thread_rng, Rng};
    let id = thread_rng()
      .sample_iter(Alphanumeric)
      .take(size)
      .collect::<Vec<u8>>();

    Node::new(
      SmolStr::from(String::from_utf8(id).unwrap()),
      thread_rng().gen(),
    )
  }

  #[test]
  fn test_node_access() {
    let mut node = random(10);
    node.set_id(SmolStr::from("test"));
    node.set_address(100);
    assert_eq!(node.id(), "test");
    assert_eq!(node.address(), &100);

    let node = node
      .cheap_clone()
      .with_id(SmolStr::from("test2"))
      .with_address(200);
    assert_eq!(node.id(), "test2");
    assert_eq!(node.address(), &200);

    let (id, address) = node.into_components();
    assert_eq!(id, "test2");
    assert_eq!(address, 200);

    let node = Node::from(("test3", 300));
    assert_eq!(*node.id(), "test3");
    assert_eq!(node.address(), &300);
    println!("{}", node);
  }

  #[cfg(feature = "transformable")]
  #[test]
  fn test_transformable() {
    use transformable::Transformable;

    let node = random(10);
    let mut buf = vec![0u8; node.encoded_len()];
    let len = node.encoded_len();
    let encoded = node.encode(&mut buf).unwrap();
    assert_eq!(len, encoded);
    let (decoded, decoded_node) = Node::<SmolStr, u64>::decode(&buf).unwrap();
    assert_eq!(decoded, len);
    assert_eq!(node, decoded_node);

    let node = random(100);
    let mut buf = vec![0u8; node.encoded_len()];
    let len = node.encoded_len();
    let encoded = node.encode(&mut buf).unwrap();
    assert_eq!(len, encoded);
    let (decoded, decoded_node) = Node::<SmolStr, u64>::decode(&buf).unwrap();
    assert_eq!(decoded, len);
    assert_eq!(node, decoded_node);
  }

  #[cfg(feature = "transformable")]
  #[test]
  fn test_transformable_io() {
    use std::io::Cursor;
    use transformable::Transformable;

    let node = random(10);
    let mut buf = Vec::new();
    let len = node.encoded_len();
    let encoded = node.encode_to_writer(&mut buf).unwrap();
    assert_eq!(len, encoded);
    let mut buf = Cursor::new(buf);
    let (len, decoded_node) = Node::<SmolStr, u64>::decode_from_reader(&mut buf).unwrap();
    assert_eq!(len, encoded);
    assert_eq!(node, decoded_node);

    let node = random(100);
    let mut buf = Vec::new();
    let len = node.encoded_len();
    let encoded = node.encode_to_writer(&mut buf).unwrap();
    assert_eq!(len, encoded);
    let mut buf = Cursor::new(buf);
    let (len, decoded_node) = Node::<SmolStr, u64>::decode_from_reader(&mut buf).unwrap();
    assert_eq!(len, encoded);
    assert_eq!(node, decoded_node);
  }

  #[cfg(all(feature = "async", feature = "transformable"))]
  #[tokio::test]
  async fn test_transformable_async_io() {
    use futures::io::Cursor;
    use transformable::Transformable;

    let node = random(10);
    let mut buf = Vec::new();
    let len = node.encoded_len();
    let encoded = node.encode_to_async_writer(&mut buf).await.unwrap();
    assert_eq!(len, encoded);
    let mut buf = Cursor::new(buf);
    let (len, decoded_node) = Node::<SmolStr, u64>::decode_from_async_reader(&mut buf)
      .await
      .unwrap();
    assert_eq!(len, encoded);
    assert_eq!(node, decoded_node);

    let node = random(100);
    let mut buf = Vec::new();
    let len = node.encoded_len();
    let encoded = node.encode_to_async_writer(&mut buf).await.unwrap();
    assert_eq!(len, encoded);
    let mut buf = Cursor::new(buf);
    let (len, decoded_node) = Node::<SmolStr, u64>::decode_from_async_reader(&mut buf)
      .await
      .unwrap();
    assert_eq!(len, encoded);
    assert_eq!(node, decoded_node);
  }

  #[cfg(feature = "serde")]
  #[test]
  fn test_serde() {
    let node = random(10);
    let serialized = serde_json::to_string(&node).unwrap();
    let deserialized: Node<SmolStr, u64> = serde_json::from_str(&serialized).unwrap();
    assert_eq!(node, deserialized);

    let node = random(100);
    let serialized = serde_json::to_string(&node).unwrap();
    let deserialized: Node<SmolStr, u64> = serde_json::from_str(&serialized).unwrap();
    assert_eq!(node, deserialized);
  }
}
