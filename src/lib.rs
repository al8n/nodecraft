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

/// `AddressResolver` trait for async.
#[cfg(feature = "resolver")]
#[cfg_attr(docsrs, doc(cfg(feature = "resolver")))]
pub mod resolver;

#[cfg(feature = "async")]
pub use futures;

pub use transformable;
pub use transformable::Transformable;

pub use cheap_clone::CheapClone;
