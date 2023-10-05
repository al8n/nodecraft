//! Crafting seamless node operations for distributed systems, which provides foundational traits for node identification and address resolution.
#![deny(missing_docs, warnings)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod address;
mod id;
pub(crate) mod utils;

pub use address::*;
pub use id::*;

/// `NodeAddressResolver` trait for async.
#[cfg(all(feature = "std", feature = "async"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "async"))))]
pub mod resolver;

#[cfg(feature = "async")]
pub use futures;

mod transformable;
pub use transformable::*;
