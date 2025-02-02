#![doc = include_str!("../README.md")]
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

pub use address::*;
pub use id::*;
pub use length_delimited;
pub use node::*;

/// `AddressResolver` trait for async.
#[cfg(feature = "resolver")]
#[cfg_attr(docsrs, doc(cfg(feature = "resolver")))]
pub mod resolver;

#[cfg(feature = "async")]
pub use futures;

pub use cheap_clone::CheapClone;
