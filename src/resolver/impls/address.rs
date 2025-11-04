use core::{time::Duration, net::SocketAddr};

use super::{super::AddressResolver, CachedSocketAddr};
use crate::address::{Domain, HostAddr};

use crossbeam_skiplist::SkipMap;

/// The options used to construct a [`AddressResolver`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HostAddrResolverOptions {
  #[cfg_attr(
    feature = "serde",
    serde(with = "humantime_serde", default = "default_record_ttl")
  )]
  record_ttl: Duration,
}

impl Default for HostAddrResolverOptions {
  fn default() -> Self {
    Self::new()
  }
}

const fn default_record_ttl() -> Duration {
  Duration::from_secs(60)
}

impl HostAddrResolverOptions {
  /// Create a new [`HostAddrResolverOptions`].
  #[inline]
  pub const fn new() -> Self {
    Self {
      record_ttl: default_record_ttl(),
    }
  }

  /// Set the DNS record ttl in builder pattern
  #[inline]
  pub const fn with_record_ttl(mut self, val: Duration) -> Self {
    self.record_ttl = val;
    self
  }

  /// Set the DNS record ttl
  #[inline]
  pub fn set_record_ttl(&mut self, val: Duration) -> &mut Self {
    self.record_ttl = val;
    self
  }

  /// Returns the DNS record ttl
  #[inline]
  pub const fn record_ttl(&self) -> Duration {
    self.record_ttl
  }
}

pub use resolver::HostAddrResolver;

#[cfg(feature = "agnostic")]
mod resolver {
  use super::*;

  use agnostic::{RuntimeLite, net::ToSocketAddrs};
  use either::Either;

  /// A resolver which supports both `domain:port` and socket address. However,
  /// it will only use [`ToSocketAddrs`](std::net::ToSocketAddrs)
  /// to resolve the address.
  ///
  /// - If you can make sure, you always play with [`SocketAddr`], you may want to
  ///   use [`SocketAddrResolver`](crate::resolver::socket_addr::SocketAddrResolver).
  /// - If you want to send DNS queries, you may want to use [`DnsResolver`](crate::resolver::dns::DnsResolver).
  ///
  /// **N.B.** If a domain contains multiple ip addresses, there is no guarantee that
  /// which one will be used. Users should make sure that the domain only contains
  /// one ip address, to make sure that [`AddressResolver`] can work properly.
  ///
  /// e.g. valid address format:
  /// 1. `www.example.com:8080` // domain
  /// 2. `[::1]:8080` // ipv6
  /// 3. `127.0.0.1:8080` // ipv4
  ///
  pub struct HostAddrResolver<R> {
    cache: SkipMap<Domain, CachedSocketAddr>,
    record_ttl: Duration,
    _marker: std::marker::PhantomData<R>,
  }

  impl<R> Default for HostAddrResolver<R> {
    fn default() -> Self {
      Self::new(Default::default())
    }
  }

  impl<R: RuntimeLite> AddressResolver for HostAddrResolver<R> {
    type Address = HostAddr;
    type ResolvedAddress = SocketAddr;
    type Error = std::io::Error;
    type Runtime = R;
    type Options = HostAddrResolverOptions;

    #[inline]
    async fn new(opts: Self::Options) -> Result<Self, Self::Error> {
      Ok(Self {
        record_ttl: opts.record_ttl,
        cache: Default::default(),
        _marker: Default::default(),
      })
    }

    async fn resolve(&self, address: &Self::Address) -> Result<SocketAddr, Self::Error> {
      match address.as_ref() {
        Either::Left(addr) => Ok(addr),
        Either::Right((port, name)) => {
          // First, check cache
          if let Some(ent) = self.cache.get(name.as_str()) {
            let val = ent.value();
            if !val.is_expired() {
              return Ok(val.val);
            } else {
              ent.remove();
            }
          }

          // Finally, try to find the socket addr locally
          let tsafe = name.clone();

          let res =
            ToSocketAddrs::<Self::Runtime>::to_socket_addrs(&(tsafe.as_str(), port)).await?;

          if let Some(addr) = res.into_iter().next() {
            self
              .cache
              .insert(name.clone(), CachedSocketAddr::new(addr, self.record_ttl));
            return Ok(addr);
          }

          Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("failed to resolve {}", name.as_str()),
          ))
        }
      }
    }
  }

  impl<R> HostAddrResolver<R> {
    /// Create a new [`HostAddrResolver`] with the given options.
    pub fn new(opts: HostAddrResolverOptions) -> Self {
      Self {
        record_ttl: opts.record_ttl,
        cache: Default::default(),
        _marker: Default::default(),
      }
    }
  }

  #[cfg(test)]
  mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dns_resolver() {
      use agnostic::tokio::TokioRuntime;

      let resolver = HostAddrResolver::<TokioRuntime>::default();
      let google_addr = HostAddr::try_from("google.com:8080").unwrap();
      let ip = resolver.resolve(&google_addr).await.unwrap();
      println!("google.com:8080 resolved to: {}", ip);
    }

    #[tokio::test]
    async fn test_dns_resolver_with_record_ttl() {
      use agnostic::tokio::TokioRuntime;

      let resolver = HostAddrResolver::<TokioRuntime>::new(
        HostAddrResolverOptions::new().with_record_ttl(Duration::from_millis(100)),
      );
      let google_addr = HostAddr::try_from("google.com:8080").unwrap();
      resolver.resolve(&google_addr).await.unwrap();
      resolver.resolve(&google_addr).await.unwrap();
      let ip_addr = HostAddr::try_from(("127.0.0.1", 8080)).unwrap();
      resolver.resolve(&ip_addr).await.unwrap();
      let dns_name = Domain::try_from("google.com").unwrap();
      assert!(
        !resolver
          .cache
          .get(dns_name.as_str())
          .unwrap()
          .value()
          .is_expired()
      );

      tokio::time::sleep(Duration::from_millis(100)).await;
      assert!(
        resolver
          .cache
          .get(dns_name.as_str())
          .unwrap()
          .value()
          .is_expired()
      );
      resolver.resolve(&google_addr).await.unwrap();

      let bad_addr = HostAddr::try_from("adasdjkljasidjaosdjaisudnaisudibasd.com:8080").unwrap();
      assert!(resolver.resolve(&bad_addr).await.is_err());
    }
  }
}

#[cfg(not(feature = "agnostic"))]
mod resolver {
  use super::*;

  /// A resolver which supports both `domain:port` and socket address. However,
  /// it will only use [`ToSocketAddrs`](std::net::ToSocketAddrs)
  /// to resolve the address.
  ///
  /// - If you can make sure, you always play with [`SocketAddr`], you may want to
  ///   use [`SocketAddrResolver`](crate::resolver::socket_addr::SocketAddrResolver).
  /// - If you want to send DNS queries, you may want to use [`DnsResolver`](crate::resolver::dns::DnsResolver).
  ///
  /// **N.B.** If a domain contains multiple ip addresses, there is no guarantee that
  /// which one will be used. Users should make sure that the domain only contains
  /// one ip address, to make sure that [`AddressResolver`] can work properly.
  ///
  /// e.g. valid address format:
  /// 1. `www.example.com:8080` // domain
  /// 2. `[::1]:8080` // ipv6
  /// 3. `127.0.0.1:8080` // ipv4
  ///
  pub struct HostAddrResolver {
    cache: SkipMap<Domain, CachedSocketAddr>,
    record_ttl: Duration,
  }

  impl AddressResolver for HostAddrResolver {
    type Address = HostAddr;
    type ResolvedAddress = SocketAddr;
    type Error = std::io::Error;
    type Options = HostAddrResolverOptions;

    #[inline]
    async fn new(opts: Self::Options) -> Result<Self, Self::Error> {
      Ok(Self {
        record_ttl: opts.record_ttl,
        cache: Default::default(),
      })
    }

    async fn resolve(&self, address: &Self::Address) -> Result<SocketAddr, Self::Error> {
      match address.as_inner() {
        Either::Left(addr) => Ok(addr),
        Either::Right((port, name)) => {
          // First, check cache
          if let Some(ent) = self.cache.get(name) {
            let val = ent.value();
            if !val.is_expired() {
              return Ok(val.val);
            } else {
              ent.remove();
            }
          }

          // Finally, try to find the socket addr locally
          let res = ToSocketAddrs::to_socket_addrs(&(name.as_str(), port))?;
          if let Some(addr) = res.into_iter().next() {
            self
              .cache
              .insert(name.clone(), CachedSocketAddr::new(addr, self.record_ttl));
            return Ok(addr);
          }

          Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("failed to resolve {}", name),
          ))
        }
      }
    }
  }

  impl Default for HostAddrResolver {
    fn default() -> Self {
      Self::new(Default::default())
    }
  }

  impl HostAddrResolver {
    /// Create a new [`HostAddrResolver`] with the given options.
    pub fn new(opts: HostAddrResolverOptions) -> Self {
      Self {
        record_ttl: opts.record_ttl,
        cache: Default::default(),
      }
    }
  }

  #[cfg(test)]
  mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dns_resolver() {
      let resolver = HostAddrResolver::default();
      let google_addr = HostAddr::try_from("google.com:8080").unwrap();
      let ip = resolver.resolve(&google_addr).await.unwrap();
      println!("google.com:8080 resolved to: {}", ip);
    }

    #[tokio::test]
    async fn test_dns_resolver_with_record_ttl() {
      let resolver = HostAddrResolver::new(
        HostAddrResolverOptions::new().with_record_ttl(Duration::from_millis(100)),
      );
      let google_addr = HostAddr::try_from("google.com:8080").unwrap();
      resolver.resolve(&google_addr).await.unwrap();
      let dns_name = Domain::try_from("google.com").unwrap();
      assert!(!resolver.cache.get(&dns_name).unwrap().value().is_expired());

      tokio::time::sleep(Duration::from_millis(100)).await;
      assert!(resolver.cache.get(&dns_name).unwrap().value().is_expired());
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_opts() {
    let opts = HostAddrResolverOptions::default();
    assert_eq!(opts.record_ttl(), default_record_ttl());
    let mut opts = opts.with_record_ttl(Duration::from_secs(10));
    assert_eq!(opts.record_ttl(), Duration::from_secs(10));
    opts.set_record_ttl(Duration::from_secs(11));
    assert_eq!(opts.record_ttl(), Duration::from_secs(11));
  }
}
