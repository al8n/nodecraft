use std::{future::Future, net::SocketAddr};

use crate::Address;

mod impls;
pub use impls::*;

#[cfg(not(feature = "agnostic"))]
/// Used to resolve a [`SocketAddr`] from a node address in async style.
pub trait AddressResolver: Send + Sync + 'static {
  /// The address type used to identify nodes.
  type Address: Address;
  /// The error type returned by the resolver.
  type Error: std::error::Error + Send + Sync + 'static;

  /// Resolves the given node address to a [`SocketAddr`].
  fn resolve(
    &self,
    address: &Self::Address,
  ) -> impl Future<Output = Result<SocketAddr, Self::Error>> + Send;
}

#[cfg(feature = "agnostic")]
/// Used to resolve a [`SocketAddr`] from a node address in async style.
pub trait AddressResolver: Send + Sync + 'static {
  /// The address type used to identify nodes.
  type Address: Address;
  /// The error type returned by the resolver.
  type Error: std::error::Error + Send + Sync + 'static;

  /// The runtime used to resolve the address.
  type Runtime: agnostic::Runtime;

  /// Resolves the given node address to a [`SocketAddr`].
  fn resolve(
    &self,
    address: &Self::Address,
  ) -> impl Future<Output = Result<SocketAddr, Self::Error>> + Send;
}
