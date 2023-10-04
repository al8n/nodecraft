use std::net::SocketAddr;

use crate::NodeAddress;

mod impls;

/// Used to resolve a [`SocketAddr`] from a node address in async style.
#[async_trait::async_trait]
pub trait NodeAddressResolver: Send + Sync + 'static {
  /// The address type used to identify nodes.
  type NodeAddress: NodeAddress;
  /// The error type returned by the resolver.
  type Error: std::error::Error + Send + Sync + 'static;

  /// The runtime used to resolve the address.
  #[cfg(feature = "agnostic")]
  #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
  type Runtime: agnostic::Runtime;

  /// Resolves the given node address to a [`SocketAddr`].
  async fn resolve(&self, address: &Self::NodeAddress) -> Result<SocketAddr, Self::Error>;
}