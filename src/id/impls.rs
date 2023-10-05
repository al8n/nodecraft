#[cfg(any(feature = "std", feature = "alloc"))]
mod id;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use id::*;

#[cfg(feature = "std")]
use std::string::String;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use ::alloc::string::String;

#[cfg(feature = "alloc")]
impl super::NodeId for String {}

#[cfg(feature = "smol_str")]
impl super::NodeId for smol_str::SmolStr {}

mod numbers;
