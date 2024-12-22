use std::future::Future;

use crate::Address;

mod impls;
use cheap_clone::CheapClone;
pub use impls::*;

#[cfg(feature = "agnostic")]
pub use agnostic_lite::*;

#[cfg(not(feature = "agnostic"))]
/// Used to resolve a [`SocketAddr`] from a node address in async style.
pub trait AddressResolver: Send + Sync + 'static {
  /// The address type used to identify nodes.
  type Address: Address;
  /// The address type returned by the resolver.
  type ResolvedAddress: CheapClone
    + core::hash::Hash
    + Eq
    + core::fmt::Debug
    + core::fmt::Display
    + Send
    + Sync
    + 'static;
  /// The error type returned by the resolver.
  type Error: core::error::Error + Send + Sync + 'static;

  /// The options type used to configure the resolver.
  type Options: Send + Sync + 'static;

  /// Creates a new resolver with the given options.
  fn new(options: Self::Options) -> impl Future<Output = Result<Self, Self::Error>> + Send
  where
    Self: Sized;

  /// Resolves the given node address to a [`SocketAddr`].
  fn resolve(
    &self,
    address: &Self::Address,
  ) -> impl Future<Output = Result<Self::ResolvedAddress, Self::Error>> + Send;
}

#[cfg(feature = "agnostic")]
/// Used to resolve a [`SocketAddr`] from a node address in async style.
pub trait AddressResolver: Send + Sync + 'static {
  /// The address type used to identify nodes.
  type Address: Address;

  /// The address type returned by the resolver.
  #[cfg(not(feature = "transformable"))]
  type ResolvedAddress: CheapClone
    + core::hash::Hash
    + Eq
    + core::fmt::Debug
    + core::fmt::Display
    + Send
    + Sync
    + 'static;

  /// The address type returned by the resolver.
  #[cfg(feature = "transformable")]
  type ResolvedAddress: CheapClone
    + core::hash::Hash
    + Eq
    + core::fmt::Debug
    + core::fmt::Display
    + transformable::Transformable
    + Send
    + Sync
    + 'static;

  /// The error type returned by the resolver.
  type Error: core::error::Error + Send + Sync + 'static;

  /// The runtime used to resolve the address.
  type Runtime: agnostic_lite::RuntimeLite;

  /// The options type used to configure the resolver.
  type Options: Send + Sync + 'static;

  /// Creates a new resolver with the given options.
  fn new(options: Self::Options) -> impl Future<Output = Result<Self, Self::Error>> + Send
  where
    Self: Sized;

  /// Resolves the given node address to a [`SocketAddr`].
  fn resolve(
    &self,
    address: &Self::Address,
  ) -> impl Future<Output = Result<Self::ResolvedAddress, Self::Error>> + Send;
}
