use core::{
  fmt::{Debug, Display},
  hash::Hash,
};

mod impls;
use cheap_clone::CheapClone;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use impls::{NodeId, NodeIdTransformError};

/// Id abstraction for distributed systems
#[cfg(feature = "transformable")]
pub trait Id:
  CheapClone + Eq + Hash + Debug + Display + transformable::Transformable + Sized + Unpin + 'static
{
}

/// Id abstraction for distributed systems
#[cfg(not(feature = "transformable"))]
pub trait Id: CheapClone + Eq + Hash + Debug + Display + Sized + Unpin + 'static {}
