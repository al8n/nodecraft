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

#[cfg(feature = "smol_str02")]
impl super::Id for smol_str02::SmolStr {}
#[cfg(feature = "smol_str03")]
impl super::Id for smol_str03::SmolStr {}

mod numbers;

#[cfg(feature = "std")]
mod net;
