use core::{fmt::Display, hash::Hash};

use crate::Transformable;

mod impls;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use impls::{Id, IdTransformableError};

/// Node id
pub trait NodeId: Clone + Eq + Hash + Display + Transformable {}
