use std::{convert::Infallible, net::SocketAddr};

use super::super::AddressResolver;

pub use resolver::SocketAddrResolver;

#[cfg(feature = "agnostic")]
mod resolver {
  use super::*;

  use agnostic::RuntimeLite;

  /// The [`AddressResolver::Address`] of the [`SocketAddrResolver`] is [`SocketAddr`].
  /// So it just returns the given address and impossible to return an error.
  ///
  /// If you want a more powerful [`AddressResolver`] implementation,
  /// please see [`DnsResolver`](crate::transport::resolver::dns::DnsResolver).
  pub struct SocketAddrResolver<R: RuntimeLite>(std::marker::PhantomData<R>);

  impl<R: RuntimeLite> Default for SocketAddrResolver<R> {
    #[inline]
    fn default() -> Self {
      Self(std::marker::PhantomData)
    }
  }

  impl<R: RuntimeLite> AddressResolver for SocketAddrResolver<R> {
    type Address = SocketAddr;
    type ResolvedAddress = SocketAddr;
    type Error = Infallible;
    type Runtime = R;
    type Options = ();

    #[inline]
    async fn new(_: Self::Options) -> Result<Self, Self::Error> {
      Ok(Self::default())
    }

    #[inline]
    async fn resolve(&self, address: &Self::Address) -> Result<Self::ResolvedAddress, Self::Error> {
      Ok(*address)
    }
  }

  #[cfg(test)]
  mod tests {
    use super::*;

    #[tokio::test]
    async fn resolve() {
      let resolver = SocketAddrResolver::<agnostic::tokio::TokioRuntime>::default();
      let address = SocketAddr::new("127.0.0.1".parse().unwrap(), 8080);
      assert_eq!(resolver.resolve(&address).await.unwrap(), address);
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

  impl AddressResolver for SocketAddrResolver {
    type Address = SocketAddr;
    type ResolvedAddress = SocketAddr;
    type Error = Infallible;
    type Options = ();

    #[inline]
    async fn new(_: Self::Options) -> Result<Self, Self::Error> {
      Ok(Self)
    }

    async fn resolve(&self, address: &Self::Address) -> Result<Self::ResolvedAddress, Self::Error> {
      Ok(*address)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[cfg(feature = "agnostic")]
  #[test]
  fn resolver() {
    let _ = SocketAddrResolver::<agnostic::tokio::TokioRuntime>::default();
  }
}