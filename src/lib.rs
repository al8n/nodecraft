#![doc = include_str!("../README.md")]
#![deny(missing_docs, warnings)]
#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc as std;

#[cfg(feature = "std")]
extern crate std;

mod address;
mod id;
mod node;

pub use address::*;
pub use id::*;
pub use node::*;

/// `AddressResolver` trait for async.
#[cfg(feature = "resolver")]
#[cfg_attr(docsrs, doc(cfg(feature = "resolver")))]
pub mod resolver;

#[cfg(feature = "async")]
pub use futures;

pub use cheap_clone::CheapClone;
