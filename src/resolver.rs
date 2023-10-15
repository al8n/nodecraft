use std::net::SocketAddr;

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
  async fn resolve(&self, address: &Self::Address) -> Result<SocketAddr, Self::Error>;
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
  async fn resolve(&self, address: &Self::Address) -> Result<SocketAddr, Self::Error>;
}
