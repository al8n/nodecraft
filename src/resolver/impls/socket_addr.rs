use std::{convert::Infallible, net::SocketAddr};

use super::super::AddressResolver;

pub use resolver::SocketAddrResolver;

#[cfg(feature = "agnostic")]
mod resolver {
  use super::*;

  use agnostic::Runtime;

  /// The [`AddressResolver::Address`] of the [`SocketAddrResolver`] is [`SocketAddr`].
  /// So it just returns the given address and impossible to return an error.
  ///
  /// If you want a more powerful [`AddressResolver`] implementation,
  /// please see [`DnsResolver`](crate::transport::resolver::dns::DnsResolver).
  pub struct SocketAddrResolver<R: Runtime>(std::marker::PhantomData<R>);

  impl<R: Runtime> Default for SocketAddrResolver<R> {
    fn default() -> Self {
      Self(std::marker::PhantomData)
    }
  }
  impl<R: Runtime> SocketAddrResolver<R> {
    /// Creates a new `SocketAddrResolver`.
    #[inline]
    pub const fn new() -> Self {
      Self(std::marker::PhantomData)
    }
  }

  impl<R: Runtime> AddressResolver for SocketAddrResolver<R> {
    type Address = SocketAddr;
    type Error = Infallible;
    type Runtime = R;

    async fn resolve(&self, address: &Self::Address) -> Result<SocketAddr, Self::Error> {
      Ok(*address)
    }
  }
}

#[cfg(not(feature = "agnostic"))]
mod resolver {
  use super::*;

  /// The [`AddressResolver::Address`] of the [`SocketAddrResolver`] is [`SocketAddr`].
  /// So it just returns the given address and impossible to return an error.
  ///
  /// If you want a more powerful [`AddressResolver`] implementation,
  /// please see [`DnsResolver`](crate::transport::resolver::dns::DnsResolver).
  pub struct SocketAddrResolver;

  impl Default for SocketAddrResolver {
    fn default() -> Self {
      Self
    }
  }

  impl SocketAddrResolver {
    /// Creates a new `SocketAddrResolver`.
    pub const fn new() -> Self {
      Self
    }
  }

  impl AddressResolver for SocketAddrResolver {
    type Address = SocketAddr;
    type Error = Infallible;

    async fn resolve(&self, address: &Self::Address) -> Result<SocketAddr, Self::Error> {
      Ok(*address)
    }
  }
}
