/// Async DNS resolver
#[cfg(feature = "dns")]
#[cfg_attr(docsrs, doc(cfg(feature = "dns")))]
pub mod dns;

/// Dummy [`SocketAddr`] resolver
#[cfg(all(feature = "std", feature = "async"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "async"))))]
pub mod socket_addr;

/// [`Address`](crate::Address) resolver, but will not send DNS query
/// and only use [`ToSocketAddrs`](std::net::ToSocketAddrs) to resolve the address.
#[cfg(all(feature = "std", feature = "async"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "async"))))]
pub mod address;

#[cfg(all(feature = "std", feature = "async"))]
struct CachedSocketAddr {
  val: std::net::SocketAddr,
  born: std::time::Instant,
  ttl: std::time::Duration,
}

#[cfg(all(feature = "std", feature = "async"))]
impl CachedSocketAddr {
  fn new(val: std::net::SocketAddr, ttl: std::time::Duration) -> Self {
    Self {
      val,
      born: std::time::Instant::now(),
      ttl,
    }
  }

  fn is_expired(&self) -> bool {
    self.born.elapsed() > self.ttl
  }
}
