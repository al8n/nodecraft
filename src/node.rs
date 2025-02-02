use core::fmt::Display;

use cheap_clone::CheapClone;

/// Node is consist of id and address, which can be used as a identifier in a distributed system.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
  feature = "rkyv",
  derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq)))]
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
  use rand::distr::Alphanumeric;
  use smol_str03::SmolStr;

  fn random(size: usize) -> Node<SmolStr, u64> {
    use rand::{rng, Rng};
    let id = rng()
      .sample_iter(Alphanumeric)
      .take(size)
      .collect::<Vec<u8>>();

    Node::new(
      SmolStr::from(String::from_utf8(id).unwrap()),
      rng().random(),
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
