use core::{
  fmt::{Debug, Display},
  hash::Hash,
};

use transformable::Transformable;

mod impls;
use cheap_clone::CheapClone;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use impls::{NodeId, NodeIdTransformError};

/// Id abstraction for distributed systems
pub trait Id:
  CheapClone + Eq + Hash + Debug + Display + Transformable + Sized + Unpin + 'static
{
}
