//! Crafting seamless node operations for distributed systems, which provides foundational traits for node identification and address resolution.
#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

mod address;
mod id;

pub use address::*;
pub use id::*;

/// `NodeAddressResolver` trait.
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub mod resolver;

/// `NodeAddressResolver` trait for async.
#[cfg(all(feature = "std", feature = "async"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "async"))))]
pub mod async_resolver;

#[cfg(feature = "async")]
pub use futures_util;

/// The type can transform its representation between structured and byte form.
#[cfg_attr(all(feature = "async", feature = "std"), async_trait::async_trait)]
pub trait Transformable {
  /// The error type returned when encoding or decoding fails.
  #[cfg(feature = "std")]
  type Error: std::error::Error + Send + Sync + 'static;

  #[cfg(not(feature = "std"))]
  type Error: core::fmt::Display + Send + Sync + 'static;

  /// Encodes the value into the given buffer for transmission.
  fn encode(&self, dst: &mut [u8]) -> Result<(), Self::Error>;

  /// Encodes the value into the given writer for transmission.
  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn encode_to_writer<W: std::io::Write>(&self, writer: &mut W) -> Result<(), Self::Error>;

  /// Encodes the value into the given async writer for transmission.
  #[cfg(all(feature = "async", feature = "std"))]
  #[cfg_attr(docsrs, doc(cfg(all(feature = "async", feature = "std"))))]
  async fn encode_to_async_writer<W: futures_util::io::AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<(), Self::Error>;

  /// Returns the encoded length of the value.
  /// This is used to pre-allocate a buffer for encoding.
  fn encoded_len(&self) -> usize;

  /// Decodes the value from the given buffer received over the wire.
  fn decode(src: &[u8]) -> Result<Self, Self::Error>
  where
    Self: Sized;
  
  /// Decodes the value from the given reader received over the wire.
  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn decode_from_reader<R: std::io::Read>(reader: &mut R) -> Result<Self, Self::Error>
  where
    Self: Sized;
  
  /// Decodes the value from the given async reader received over the wire.
  #[cfg(all(feature = "async", feature = "std"))]
  #[cfg_attr(docsrs, doc(cfg(all(feature = "async", feature = "std"))))]
  async fn decode_from_async_reader<R: futures_util::io::AsyncRead + Unpin>(reader: &mut R) -> Result<Self, Self::Error>
  where
    Self: Sized;
}
