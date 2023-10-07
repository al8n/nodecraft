#[cfg(any(feature = "std", feature = "alloc"))]
mod id;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use id::*;

#[cfg(feature = "std")]
use std::{boxed::Box, string::String, sync::Arc};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use ::alloc::{boxed::Box, string::String, sync::Arc};

#[cfg(feature = "alloc")]
impl super::Id for String {}

#[cfg(feature = "alloc")]
impl super::Id for Box<str> {}

#[cfg(feature = "alloc")]
impl super::Id for Arc<str> {}

#[cfg(feature = "smol_str")]
impl super::Id for smol_str::SmolStr {}

mod numbers;
pub use numbers::NumberIdTransformableError;
