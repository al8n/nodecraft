use core::time::Duration;
use std::{io, net::SocketAddr};

pub use agnostic::{
  dns::{AsyncConnectionProvider, Dns, ResolverConfig, ResolverOpts},
  net::Net,
};
use agnostic::{net::ToSocketAddrs, Runtime};
use crossbeam_skiplist::SkipMap;

use super::{super::AddressResolver, CachedSocketAddr};
use crate::{Domain, Kind, HostAddr};

#[derive(Debug, thiserror::Error)]
enum ResolveErrorKind {
  #[error("cannot resolve an ip address for {0}")]
  NotFound(Domain),
  #[error(transparent)]
  Resolve(#[from] hickory_resolver::error::ResolveError),
}

/// The error type for errors that get returned when resolving fails
#[derive(Debug)]
#[repr(transparent)]
pub struct ResolveError(ResolveErrorKind);

impl core::fmt::Display for ResolveError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
    self.0.fmt(f)
  }
}

impl core::error::Error for ResolveError {}

impl From<ResolveErrorKind> for ResolveError {
  fn from(value: ResolveErrorKind) -> Self {
    Self(value)
  }
}

/// Errors that can occur when resolving an address.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// Returns when there is an io error
  #[error(transparent)]
  IO(#[from] io::Error),
  /// Returns when there is an error when resolving an address
  #[error(transparent)]
  Resolve(#[from] ResolveError),
}

/// The options used to configure the DNS
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DnsOptions {
  resolver_opts: ResolverOpts,
  resolver_config: ResolverConfig,
}

const fn default_record_ttl() -> Duration {
  Duration::from_secs(60)
}

impl DnsOptions {
  /// Create a new [`DnsResolverOptions`] with the default DNS configurations.
  pub fn new() -> Self {
    Self {
      resolver_opts: ResolverOpts::default(),
      resolver_config: ResolverConfig::default(),
    }
  }

  /// Set the default dns configuration in builder pattern
  pub fn with_resolver_config(mut self, c: ResolverConfig) -> Self {
    self.resolver_config = c;
    self
  }

  /// Set the default dns configuration
  pub fn set_resolver_config(&mut self, c: ResolverConfig) -> &mut Self {
    self.resolver_config = c;
    self
  }

  /// Returns the resolver configuration
  pub fn resolver_config(&self) -> &ResolverConfig {
    &self.resolver_config
  }

  /// Set the default resolver options in builder pattern
  pub fn with_resolver_opts(mut self, o: ResolverOpts) -> Self {
    self.resolver_opts = o;
    self
  }

  /// Set the default resolver options
  pub fn set_resolver_opts(&mut self, o: ResolverOpts) -> &mut Self {
    self.resolver_opts = o;
    self
  }

  /// Returns the resolver options
  pub fn resolver_opts(&self) -> &ResolverOpts {
    &self.resolver_opts
  }
}

impl Default for DnsOptions {
  fn default() -> Self {
    Self::new()
  }
}

/// The options used to construct a [`DnsResolver`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DnsResolverOptions {
  #[cfg_attr(feature = "serde", serde(default = "default_record_ttl"))]
  record_ttl: Duration,
  dns: Option<DnsOptions>,
}

impl Default for DnsResolverOptions {
  fn default() -> Self {
    Self::new()
  }
}

impl DnsResolverOptions {
  /// Create a new [`DnsResolverOptions`] with the default DNS configurations.
  #[inline]
  pub fn new() -> Self {
    Self {
      record_ttl: default_record_ttl(),
      dns: Some(DnsOptions::default()),
    }
  }

  /// Set the default record ttl in builder pattern
  #[inline]
  pub const fn with_record_ttl(mut self, ttl: Duration) -> Self {
    self.record_ttl = ttl;
    self
  }

  /// Set the default record ttl
  #[inline]
  pub fn set_record_ttl(&mut self, ttl: Duration) -> &mut Self {
    self.record_ttl = ttl;
    self
  }

  /// Returns the record ttl
  #[inline]
  pub const fn record_ttl(&self) -> Duration {
    self.record_ttl
  }

  /// Set the default dns configuration in builder pattern
  #[inline]
  pub fn with_dns(mut self, dns: Option<DnsOptions>) -> Self {
    self.dns = dns;
    self
  }

  /// Set the default dns configuration
  #[inline]
  pub fn set_dns(&mut self, dns: Option<DnsOptions>) -> &mut Self {
    self.dns = dns;
    self
  }

  /// Returns the dns configuration
  #[inline]
  pub const fn dns(&self) -> Option<&DnsOptions> {
    self.dns.as_ref()
  }
}

/// A resolver which supports both `domain:port` and socket address.
///
/// - If you can make sure, you always play with [`SocketAddr`], you may want to
///   use [`SocketAddrResolver`](crate::resolver::socket_addr::SocketAddrResolver).
/// - If you do not want to send DNS queries, you may want to use [`AddressResolver`](crate::resolver::address::AddressResolver).
///
/// **N.B.** If a domain contains multiple ip addresses, there is no guarantee that
/// which one will be used. Users should make sure that the domain only contains
/// one ip address, to make sure that [`DnsResolver`] can work properly.
///
/// e.g. valid address format:
/// 1. `www.example.com:8080` // domain
/// 2. `[::1]:8080` // ipv6
/// 3. `127.0.0.1:8080` // ipv4
pub struct DnsResolver<R: Runtime> {
  dns: Option<Dns<R::Net>>,
  record_ttl: Duration,
  cache: SkipMap<Domain, CachedSocketAddr>,
}

impl<R: Runtime> AddressResolver for DnsResolver<R> {
  type Address = HostAddr;
  type Error = Error;
  type ResolvedAddress = SocketAddr;
  type Runtime = R;
  type Options = DnsResolverOptions;

  async fn new(opts: Self::Options) -> Result<Self, Self::Error>
  where
    Self: Sized,
  {
    let dns = if let Some(opts) = opts.dns {
      Some(Dns::new(
        opts.resolver_config,
        opts.resolver_opts,
        AsyncConnectionProvider::new(),
      ))
    } else {
      None
    };
    Ok(Self {
      dns,
      record_ttl: opts.record_ttl,
      cache: Default::default(),
    })
  }

  async fn resolve(&self, address: &Self::Address) -> Result<Self::ResolvedAddress, Self::Error> {
    match &address.kind {
      Kind::Ip(ip) => Ok(SocketAddr::new(*ip, address.port)),
      Kind::Domain(name) => {
        // First, check cache
        if let Some(ent) = self.cache.get(name.as_str()) {
          let val = ent.value();
          if !val.is_expired() {
            return Ok(val.val);
          } else {
            ent.remove();
          }
        }

        // Second, TCP lookup ip address
        if let Some(ref dns) = self.dns {
          if let Some(ip) = dns
            .lookup_ip(name.fqdn_str())
            .await
            .map_err(|e| ResolveError::from(ResolveErrorKind::from(e)))?
            .into_iter()
            .next()
          {
            let addr = SocketAddr::new(ip, address.port);
            self
              .cache
              .insert(name.clone(), CachedSocketAddr::new(addr, self.record_ttl));
            return Ok(addr);
          }
        }

        // Finally, try to find the socket addr locally
        let port = address.port;
        let tsafe = name.clone();

        let res = ToSocketAddrs::<R>::to_socket_addrs(&(tsafe.as_str(), port)).await?;

        if let Some(addr) = res.into_iter().next() {
          self
            .cache
            .insert(name.clone(), CachedSocketAddr::new(addr, self.record_ttl));
          return Ok(addr);
        }

        Err(Error::Resolve(ResolveError(ResolveErrorKind::NotFound(
          name.clone(),
        ))))
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_dns_resolver() {
    use agnostic::tokio::TokioRuntime;

    let resolver = DnsResolver::<TokioRuntime>::new(Default::default())
      .await
      .unwrap();
    let google_addr = HostAddr::try_from("google.com:8080").unwrap();
    let ip = resolver.resolve(&google_addr).await.unwrap();
    println!("google.com:8080 resolved to: {}", ip);
  }

  #[tokio::test]
  async fn test_dns_resolver_with_record_ttl() {
    use agnostic::tokio::TokioRuntime;

    let resolver = DnsResolver::<TokioRuntime>::new(
      DnsResolverOptions::default().with_record_ttl(Duration::from_millis(100)),
    )
    .await
    .unwrap();
    let google_addr = HostAddr::try_from("google.com:8080").unwrap();
    resolver.resolve(&google_addr).await.unwrap();
    let dns_name = Domain::try_from("google.com").unwrap();
    assert!(!resolver
      .cache
      .get(dns_name.as_str())
      .unwrap()
      .value()
      .is_expired());

    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(resolver
      .cache
      .get(dns_name.as_str())
      .unwrap()
      .value()
      .is_expired());
  }

  #[tokio::test]
  async fn test_dns_resolver_without_dns() {
    use agnostic::tokio::TokioRuntime;

    let resolver = DnsResolver::<TokioRuntime>::new(
      DnsResolverOptions::default()
        .with_dns(None)
        .with_record_ttl(Duration::from_millis(100)),
    )
    .await
    .unwrap();
    let google_addr = HostAddr::try_from("google.com:8080").unwrap();
    resolver.resolve(&google_addr).await.unwrap();
    resolver.resolve(&google_addr).await.unwrap();
    let ip_addr = HostAddr::try_from(("127.0.0.1", 8080)).unwrap();
    resolver.resolve(&ip_addr).await.unwrap();
    let dns_name = Domain::try_from("google.com").unwrap();
    assert!(!resolver
      .cache
      .get(dns_name.as_str())
      .unwrap()
      .value()
      .is_expired());

    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(resolver
      .cache
      .get(dns_name.as_str())
      .unwrap()
      .value()
      .is_expired());
    resolver.resolve(&google_addr).await.unwrap();

    let err = ResolveError::from(ResolveErrorKind::NotFound(dns_name.clone()));
    println!("{err}");
    println!("{err:?}");

    let bad_addr = HostAddr::try_from("adasdjkljasidjaosdjaisudnaisudibasd.com:8080").unwrap();
    assert!(resolver.resolve(&bad_addr).await.is_err());
  }

  #[test]
  fn test_opts() {
    let opts = DnsOptions::new();
    let opts = opts.with_resolver_config(Default::default());
    opts.resolver_config();
    let mut opts = opts.with_resolver_opts(Default::default());
    opts.resolver_opts();
    opts.set_resolver_config(Default::default());
    opts.set_resolver_opts(Default::default());

    let mut opts = DnsResolverOptions::new().with_dns(Some(opts));
    opts.dns();
    opts.set_dns(Some(Default::default()));
    opts.set_record_ttl(Duration::from_secs(100));
    opts.record_ttl();
  }
}
