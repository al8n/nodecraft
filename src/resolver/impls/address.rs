use core::time::Duration;
use std::net::{SocketAddr, ToSocketAddrs};

use super::{super::AddressResolver, CachedSocketAddr};
use crate::{Kind, NodeAddress};

use crossbeam_skiplist::SkipMap;
use smol_str::SmolStr;

/// The options used to construct a [`AddressResolver`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeAddressResolverOptions {
  #[cfg_attr(
    feature = "serde",
    serde(with = "humantime_serde", default = "default_record_ttl")
  )]
  record_ttl: Duration,
}

impl Default for NodeAddressResolverOptions {
  fn default() -> Self {
    Self {
      record_ttl: default_record_ttl(),
    }
  }
}

const fn default_record_ttl() -> Duration {
  Duration::from_secs(60)
}

impl NodeAddressResolverOptions {
  /// Create a new [`NodeAddressResolverOptions`].
  pub fn new() -> Self {
    Self::default()
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

pub use resolver::NodeAddressResolver;

#[cfg(feature = "agnostic")]
mod resolver {
  use super::*;

  use agnostic::Runtime;

  /// A resolver which supports both `domain:port` and socket address. However,
  /// it will only use [`ToSocketAddrs`](std::net::ToSocketAddrs)
  /// to resolve the address.
  ///
  /// - If you can make sure, you always play with [`SocketAddr`], you may want to
  /// use [`SocketAddrResolver`](crate::resolver::socket_addr::SocketAddrResolver).
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
  pub struct NodeAddressResolver<R: Runtime> {
    cache: SkipMap<SmolStr, CachedSocketAddr>,
    record_ttl: Duration,
    _marker: std::marker::PhantomData<R>,
  }

  impl<R: Runtime> Default for NodeAddressResolver<R> {
    fn default() -> Self {
      Self::new(Default::default())
    }
  }

  impl<R: Runtime> AddressResolver for NodeAddressResolver<R> {
    type Address = NodeAddress;
    type Error = std::io::Error;
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

          Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("failed to resolve {}", original),
          ))
        }
      }
    }
  }

  impl<R: Runtime> NodeAddressResolver<R> {
    /// Create a new [`NodeAddressResolver`] with the given options.
    pub fn new(opts: NodeAddressResolverOptions) -> Self {
      Self {
        record_ttl: opts.record_ttl,
        cache: Default::default(),
        _marker: Default::default(),
      }
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
  /// use [`SocketAddrResolver`](crate::resolver::socket_addr::SocketAddrResolver).
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
  pub struct NodeAddressResolver {
    cache: SkipMap<SmolStr, CachedSocketAddr>,
    record_ttl: Duration,
  }

  impl AddressResolver for NodeAddressResolver {
    type Address = NodeAddress;
    type Error = std::io::Error;

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

          // Finally, try to find the socket addr locally
          let res = ToSocketAddrs::to_socket_addrs(&(safe.as_str(), address.port))?;
          if let Some(addr) = res.into_iter().next() {
            self
              .cache
              .insert(safe.clone(), CachedSocketAddr::new(addr, self.record_ttl));
            return Ok(addr);
          }

          Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("failed to resolve {}", original),
          ))
        }
      }
    }
  }

  impl Default for NodeAddressResolver {
    fn default() -> Self {
      Self::new(Default::default())
    }
  }

  impl NodeAddressResolver {
    /// Create a new [`NodeAddressResolver`] with the given options.
    pub fn new(opts: NodeAddressResolverOptions) -> Self {
      Self {
        record_ttl: opts.record_ttl,
        cache: Default::default(),
      }
    }
  }
}
