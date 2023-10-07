use core::{fmt::Display, hash::Hash};

use crate::Transformable;

mod impls;
pub use impls::NumberIdTransformableError;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use impls::{NodeId, NodeIdTransformableError};

/// Id abstraction for distributed systems
pub trait Id: Clone + Eq + Hash + Display + Transformable + Sized + 'static {}
