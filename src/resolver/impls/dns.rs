use core::time::Duration;
use std::{
  io,
  net::{SocketAddr, ToSocketAddrs},
};

pub use agnostic::net::dns::*;
use agnostic::Runtime;
use crossbeam_skiplist::SkipMap;

use super::{super::AddressResolver, CachedSocketAddr};
use crate::{DnsName, Kind, NodeAddress};

#[derive(Debug, thiserror::Error)]
enum ResolveErrorKind {
  #[error("cannot resolve an ip address for {0}")]
  NotFound(DnsName),
  #[error("{0}")]
  Resolve(#[from] hickory_resolver::error::ResolveError),
}

/// The error type for errors that get returned when resolving fails
#[derive(Debug)]
#[repr(transparent)]
pub struct ResolveError(ResolveErrorKind);

impl core::fmt::Display for ResolveError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.0.fmt(f)
  }
}

impl std::error::Error for ResolveError {}

impl From<ResolveErrorKind> for ResolveError {
  fn from(value: ResolveErrorKind) -> Self {
    Self(value)
  }
}

/// Errors that can occur when resolving an address.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// Returns when there is an io error
  #[error("{0}")]
  IO(#[from] io::Error),
  /// Returns when there is an error when resolving an address
  #[error("{0}")]
  Resolve(#[from] ResolveError),
}

/// The options used to construct a [`DnsResolver`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DnsResolverOptions {
  resolver_opts: ResolverOpts,
  resolver_config: ResolverConfig,
}

const fn default_record_ttl() -> Duration {
  Duration::from_secs(60)
}

impl DnsResolverOptions {
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
  pub fn set_resolver_config(&mut self, c: ResolverConfig) {
    self.resolver_config = c;
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
  pub fn set_resolver_opts(&mut self, o: ResolverOpts) {
    self.resolver_opts = o;
  }

  /// Returns the resolver options
  pub fn resolver_opts(&self) -> &ResolverOpts {
    &self.resolver_opts
  }
}

impl Default for DnsResolverOptions {
  fn default() -> Self {
    Self::new()
  }
}

/// A resolver which supports both `domain:port` and socket address.
///
/// - If you can make sure, you always play with [`SocketAddr`], you may want to
/// use [`SocketAddrResolver`](crate::resolver::socket_addr::SocketAddrResolver).
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
///
pub struct DnsResolver<R: Runtime> {
  dns: Option<Dns<R>>,
  record_ttl: Duration,
  cache: SkipMap<DnsName, CachedSocketAddr>,
}

impl<R: Runtime> DnsResolver<R> {
  /// Create a new [`DnsResolver`] with the given options.
  pub fn new(opts: Option<DnsResolverOptions>) -> Result<Self, Error> {
    let dns = if let Some(opts) = opts {
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
      record_ttl: default_record_ttl(),
      cache: Default::default(),
    })
  }

  /// Create a new [`DnsResolver`] with the given options and the ttl for the record.
  pub fn with_record_ttl(
    opts: Option<DnsResolverOptions>,
    record_ttl: Duration,
  ) -> Result<Self, Error> {
    let dns = if let Some(opts) = opts {
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
      record_ttl,
      cache: Default::default(),
    })
  }
}

impl<R: Runtime> AddressResolver for DnsResolver<R> {
  type Address = NodeAddress;
  type Error = Error;
  type ResolvedAddress = SocketAddr;
  type Runtime = R;

  async fn resolve(&self, address: &Self::Address) -> Result<Self::ResolvedAddress, Self::Error> {
    match &address.kind {
      Kind::Ip(ip) => Ok(SocketAddr::new(*ip, address.port)),
      Kind::Dns(name) => {
        // First, check cache
        if let Some(ent) = self.cache.get(name) {
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
            .lookup_ip(name.terminate_str())
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
        let (tx, rx) = futures::channel::oneshot::channel();
        let port = address.port;
        let tsafe = name.clone();

        R::spawn_blocking_detach(move || {
          if tx
            .send(ToSocketAddrs::to_socket_addrs(&(tsafe.as_str(), port)))
            .is_err()
          {
            #[cfg(feature = "tracing")]
            tracing::warn!(
              target = "nodecraft.resolver.dns",
              "failed to resolve {} to socket address: receiver dropped",
              tsafe,
            );
          }
        });

        let res = rx
          .await
          .map_err(|e| std::io::Error::new(std::io::ErrorKind::BrokenPipe, e))??;
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

    let resolver = DnsResolver::<TokioRuntime>::new(Some(Default::default())).unwrap();
    let google_addr = NodeAddress::try_from("google.com:8080").unwrap();
    let ip = resolver.resolve(&google_addr).await.unwrap();
    println!("google.com:8080 resolved to: {}", ip);
  }

  #[tokio::test]
  async fn test_dns_resolver_with_record_ttl() {
    use agnostic::tokio::TokioRuntime;

    let resolver = DnsResolver::<TokioRuntime>::with_record_ttl(
      Some(Default::default()),
      Duration::from_millis(100),
    )
    .unwrap();
    let google_addr = NodeAddress::try_from("google.com:8080").unwrap();
    resolver.resolve(&google_addr).await.unwrap();
    let dns_name = DnsName::try_from("google.com").unwrap();
    assert!(!resolver.cache.get(&dns_name).unwrap().value().is_expired());

    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(resolver.cache.get(&dns_name).unwrap().value().is_expired());
  }
}
