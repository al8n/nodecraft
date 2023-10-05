use std::{convert::Infallible, net::SocketAddr};

use super::super::NodeAddressResolver;

#[cfg(feature = "agnostic")]
pub use ag::SocketAddrResolver;

#[cfg(feature = "agnostic")]
mod ag {
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
pub use nag::SocketAddrResolver;

#[cfg(not(feature = "agnostic"))]
mod nag {
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
