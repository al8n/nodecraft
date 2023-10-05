/// Async DNS resolver
#[cfg(feature = "dns")]
#[cfg_attr(docsrs, doc(cfg(feature = "dns")))]
pub mod dns;

/// Dummy [`SocketAddr`] resolver
#[cfg(all(feature = "std", feature = "async"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "async"))))]
pub mod socket_addr;
