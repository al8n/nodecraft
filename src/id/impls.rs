#[cfg(any(feature = "std", feature = "alloc"))]
mod id;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use id::*;

#[cfg(feature = "std")]
use std::sync::Arc;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use ::alloc::sync::Arc;

#[cfg(feature = "alloc")]
impl super::Id for Arc<str> {}

#[cfg(feature = "smol_str")]
impl super::Id for smol_str::SmolStr {}

mod numbers;
pub use numbers::NumberIdTransformableError;
