//! Crafting seamless node operations for distributed systems, which provides foundational traits for node identification and address resolution.
#![deny(missing_docs, warnings)]
#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(any(feature = "std", test))]
extern crate std;

mod address;
mod id;
mod node;
pub(crate) mod utils;

pub use address::*;
pub use id::*;
pub use node::*;

/// `AddressResolver` trait for async.
#[cfg(feature = "resolver")]
#[cfg_attr(docsrs, doc(cfg(feature = "resolver")))]
pub mod resolver;

#[cfg(feature = "async")]
pub use futures;

#[cfg(feature = "transformable")]
pub use transformable::{self, Transformable};

pub use cheap_clone::CheapClone;
