use std::{convert::Infallible, net::SocketAddr};

use super::super::NodeAddressResolver;

pub use resolver::SocketAddrResolver;

#[cfg(feature = "agnostic")]
mod resolver {
  use super::*;

  use agnostic::Runtime;

  /// The [`NodeAddressResolver::NodeAddress`] of the [`SocketAddrResolver`] is [`SocketAddr`].
  /// So it just returns the given address and impossible to return an error.
  ///
  /// If you want a more powerful [`NodeAddressResolver`] implementation,
  /// please see [`DnsResolver`](crate::transport::resolver::dns::DnsResolver).
  pub struct SocketAddrResolver<R: Runtime>(std::marker::PhantomData<R>);

  #[async_trait::async_trait]
  impl<R: Runtime> NodeAddressResolver for SocketAddrResolver<R> {
    type NodeAddress = SocketAddr;
    type Error = Infallible;
    type Runtime = R;

    async fn resolve(&self, address: &Self::NodeAddress) -> Result<SocketAddr, Self::Error> {
      Ok(*address)
    }
  }
}

#[cfg(not(feature = "agnostic"))]
mod resolver {
  use super::*;

  /// The [`NodeAddressResolver::NodeAddress`] of the [`SocketAddrResolver`] is [`SocketAddr`].
  /// So it just returns the given address and impossible to return an error.
  ///
  /// If you want a more powerful [`NodeAddressResolver`] implementation,
  /// please see [`DnsResolver`](crate::transport::resolver::dns::DnsResolver).
  pub struct SocketAddrResolver;

  #[async_trait::async_trait]
  impl NodeAddressResolver for SocketAddrResolver {
    type NodeAddress = SocketAddr;
    type Error = Infallible;

    async fn resolve(&self, address: &Self::NodeAddress) -> Result<SocketAddr, Self::Error> {
      Ok(*address)
    }
  }
}
