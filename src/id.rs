use core::{
  fmt::{Debug, Display},
  hash::Hash,
};

mod impls;
use cheap_clone::CheapClone;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use impls::{NodeId, ParseNodeIdError};

/// Id abstraction for distributed systems
pub trait Id: CheapClone + Eq + Ord + Hash + Debug + Display + Sized + Unpin + 'static {}

impl<T> Id for T where T: CheapClone + Eq + Ord + Hash + Debug + Display + Sized + Unpin + 'static {}
