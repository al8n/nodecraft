use core::time::Duration;
use std::{
  io,
  net::{SocketAddr, ToSocketAddrs},
  path::PathBuf,
};

use agnostic::{
  net::dns::{read_resolv_conf, AsyncConnectionProvider, Dns},
  Runtime,
};

use crossbeam_skiplist::SkipMap;

use smol_str::SmolStr;

use crate::{Kind, NodeAddress};

use super::{super::AddressResolver, CachedSocketAddr};

#[derive(Debug, thiserror::Error)]
enum ResolveErrorKind {
  #[error("cannot resolve an ip address for {0}")]
  NotFound(SmolStr),
  #[error("{0}")]
  Resolve(#[from] trust_dns_resolver::error::ResolveError),
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
  dns_config_path: Option<PathBuf>,

  #[cfg_attr(
    feature = "serde",
    serde(with = "humantime_serde", default = "default_record_ttl")
  )]
  record_ttl: Duration,
}

const fn default_record_ttl() -> Duration {
  Duration::from_secs(60)
}

impl DnsResolverOptions {
  /// Create a new [`DnsResolverOptions`] with the default DNS configuration path.
  ///
  /// Default DNS configuration path:
  /// 1. `Some(PathBuf::from("/etc/resolv.conf"))` for UNIX
  /// 2. `None` on other OS
  pub fn new() -> Self {
    Self::default()
  }

  /// Set the default dns configuration file path in builder pattern
  pub fn with_dns_config(mut self, p: Option<PathBuf>) -> Self {
    self.dns_config_path = p;
    self
  }

  /// Set the default dns configuration file path
  pub fn set_dns_config(&mut self, p: Option<PathBuf>) {
    self.dns_config_path = p;
  }

  /// Returns the default dns configuration file path, if any.
  pub fn dns_config(&self) -> Option<&PathBuf> {
    self.dns_config_path.as_ref()
  }

  /// Set the DNS record ttl in builder pattern
  pub const fn with_record_ttl(mut self, val: Duration) -> Self {
    self.record_ttl = val;
    self
  }

  /// Set the DNS record ttl
  pub fn set_record_ttl(&mut self, val: Duration) {
    self.record_ttl = val;
  }

  /// Returns the DNS record ttl
  pub const fn record_ttl(&self) -> Duration {
    self.record_ttl
  }
}

#[cfg(unix)]
impl Default for DnsResolverOptions {
  fn default() -> Self {
    Self {
      dns_config_path: Some(PathBuf::from("/etc/resolv.conf")),
      record_ttl: default_record_ttl(),
    }
  }
}

#[cfg(not(unix))]
impl Default for DnsResolverOptions {
  fn default() -> Self {
    Self {
      dns_config_path: None,
      record_ttl: default_record_ttl(),
    }
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
  cache: SkipMap<SmolStr, CachedSocketAddr>,
}

impl<R: Runtime> DnsResolver<R> {
  /// Create a new [`DnsResolver`] with the given options.
  pub fn new(opts: DnsResolverOptions) -> Result<Self, Error> {
    let dns = if let Some(ref path) = opts.dns_config_path {
      let (config, options) = read_resolv_conf(path)?;
      if config.name_servers().is_empty() {
        #[cfg(feature = "tracing")]
        tracing::warn!(
          target = "nodecraft.resolver.dns",
          "no DNS servers found in {}",
          path.display()
        );

        None
      } else {
        Some(Dns::new(config, options, AsyncConnectionProvider::new()))
      }
    } else {
      #[cfg(feature = "tracing")]
      tracing::warn!(
        target = "nodecraft.resolver.dns",
        "no default DNS configuration file",
      );
      None
    };

    Ok(Self {
      dns,
      record_ttl: opts.record_ttl,
      cache: Default::default(),
    })
  }
}

#[async_trait::async_trait]
impl<R: Runtime> AddressResolver for DnsResolver<R> {
  type Address = NodeAddress;
  type Error = Error;
  type Runtime = R;

  async fn resolve(&self, address: &Self::Address) -> Result<SocketAddr, Self::Error> {
    match &address.kind {
      Kind::Ip(ip) => Ok(SocketAddr::new(*ip, address.port)),
      Kind::Domain { safe, original } => {
        // First, check cache
        if let Some(ent) = self.cache.get(safe) {
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
            .lookup_ip(safe.as_str())
            .await
            .map_err(|e| ResolveError::from(ResolveErrorKind::from(e)))?
            .into_iter()
            .next()
          {
            let addr = SocketAddr::new(ip, address.port);
            self
              .cache
              .insert(safe.clone(), CachedSocketAddr::new(addr, self.record_ttl));
            return Ok(addr);
          }
        }

        // Finally, try to find the socket addr locally
        let (tx, rx) = futures::channel::oneshot::channel();
        let port = address.port;
        let tsafe = safe.clone();

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
            .insert(safe.clone(), CachedSocketAddr::new(addr, self.record_ttl));
          return Ok(addr);
        }

        Err(Error::Resolve(ResolveError(ResolveErrorKind::NotFound(
          original.clone(),
        ))))
      }
    }
  }
}
