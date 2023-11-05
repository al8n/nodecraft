use core::{
  fmt::{Debug, Display},
  hash::Hash,
};

use crate::Transformable;

mod impls;
use cheap_clone::CheapClone;
pub use impls::NumberIdTransformableError;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use impls::{NodeId, NodeIdTransformableError};

/// Id abstraction for distributed systems
pub trait Id: CheapClone + Eq + Hash + Debug + Display + Transformable + Sized + 'static {}
