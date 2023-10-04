use std::net::SocketAddr;

use crate::NodeAddress;

mod impls;


/// Used to resolve a [`SocketAddr`] from a node address.
pub trait NodeAddressResolver: Send + Sync + 'static {
  /// The address type used to identify nodes.
  type NodeAddress: NodeAddress;
  /// The error type returned by the resolver.
  type Error: std::error::Error + Send + Sync + 'static;

  /// Resolves the given node address to a [`SocketAddr`].
  fn resolve(&self, address: &Self::NodeAddress) -> Result<SocketAddr, Self::Error>;
}


