use std::{
  net::{IpAddr, SocketAddr},
  str::FromStr,
};

mod domain;
pub(crate) use domain::{Domain, InvalidDomainError};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
  feature = "rkyv",
  derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(
  feature = "rkyv",
  rkyv(compare(PartialEq), derive(PartialEq, Eq, PartialOrd, Ord, Hash))
)]
pub(crate) enum Kind {
  Ip(IpAddr),
  Domain(Domain),
}

impl PartialOrd for Kind {
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Kind {
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    match (self, other) {
      (Self::Ip(a), Self::Ip(b)) => a.cmp(b),
      (Self::Domain(a), Self::Domain(b)) => a.cmp(b),
      (Self::Ip(_), Self::Domain(_)) => core::cmp::Ordering::Less,
      (Self::Domain(_), Self::Ip(_)) => core::cmp::Ordering::Greater,
    }
  }
}

/// An error which can be returned when parsing a [`NodeAddress`].
#[derive(Debug, thiserror::Error)]
pub enum ParseNodeAddressError {
  /// Returned if the provided str is missing port.
  #[error("address is missing port")]
  MissingPort,
  /// Returned if the provided str is not a valid address.
  #[error("invalid DNS name {0}")]
  InvalidDomain(InvalidDomainError),
  /// Returned if the provided str is not a valid port.
  #[error("invalid port: {0}")]
  InvalidPort(#[from] core::num::ParseIntError),
}

/// An error which can be returned when encoding/decoding a [`NodeAddress`].
#[derive(Debug, thiserror::Error)]
pub enum NodeAddressError {
  /// Returned if the provided buffer is too small.
  #[error(
    "buffer is too small, use `Address::encoded_len` to pre-allocate a buffer with enough space"
  )]
  EncodeBufferTooSmall,
  /// Returned if fail to parsing the domain address.
  #[error(transparent)]
  ParseNodeAddressError(#[from] ParseNodeAddressError),
  /// Returned if the provided bytes is corrupted.
  #[error("{0}")]
  Corrupted(&'static str),
  /// Returned if the provided bytes contains an unknown address tag.
  #[error("unknown address tag: {0}")]
  UnknownAddressTag(u8),
  /// Returned if the provided bytes is not a valid utf8 string.
  #[error(transparent)]
  Utf8Error(#[from] core::str::Utf8Error),
}

/// A node address which supports both `domain:port` and socket address.
///
/// e.g. Valid format
/// 1. `www.example.com:8080`
/// 2. `[::1]:8080`
/// 3. `127.0.0.1:8080`
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[cfg_attr(
  feature = "rkyv",
  derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(
  feature = "rkyv",
  rkyv(compare(PartialEq), derive(PartialEq, Eq, PartialOrd, Ord, Hash))
)]
pub struct NodeAddress {
  pub(crate) kind: Kind,
  pub(crate) port: u16,
}

impl PartialOrd for NodeAddress {
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for NodeAddress {
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    match self.kind.cmp(&other.kind) {
      core::cmp::Ordering::Equal => self.port.cmp(&other.port),
      ord => ord,
    }
  }
}

impl core::fmt::Display for NodeAddress {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match &self.kind {
      Kind::Ip(addr) => write!(f, "{}", SocketAddr::new(*addr, self.port)),
      Kind::Domain(name) => write!(f, "{}:{}", name.as_str(), self.port),
    }
  }
}

#[cfg(feature = "serde")]
impl serde::Serialize for NodeAddress {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    match &self.kind {
      Kind::Ip(ip) => SocketAddr::new(*ip, self.port)
        .to_string()
        .serialize(serializer),
      Kind::Domain(name) => {
        let s = format!("{}:{}", name.as_str(), self.port);
        s.serialize(serializer)
      }
    }
  }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for NodeAddress {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    <&str as serde::Deserialize>::deserialize(deserializer)
      .and_then(|s| Self::from_str(s).map_err(<D::Error as serde::de::Error>::custom))
  }
}

impl From<SocketAddr> for NodeAddress {
  fn from(addr: SocketAddr) -> Self {
    Self {
      kind: Kind::Ip(addr.ip()),
      port: addr.port(),
    }
  }
}

impl From<(IpAddr, u16)> for NodeAddress {
  fn from(addr: (IpAddr, u16)) -> Self {
    Self {
      kind: Kind::Ip(addr.0),
      port: addr.1,
    }
  }
}

impl TryFrom<String> for NodeAddress {
  type Error = ParseNodeAddressError;

  fn try_from(s: String) -> Result<Self, Self::Error> {
    Self::from_str(s.as_str())
  }
}

impl TryFrom<&str> for NodeAddress {
  type Error = ParseNodeAddressError;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    Self::from_str(value)
  }
}

impl FromStr for NodeAddress {
  type Err = ParseNodeAddressError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let res: Result<SocketAddr, _> = s.parse();
    match res {
      Ok(addr) => Ok(addr.into()),
      Err(_) => {
        let res: Result<IpAddr, _> = s.parse();
        match res {
          Ok(_) => Err(ParseNodeAddressError::MissingPort),
          Err(_) => {
            let Some((domain, port)) = s.rsplit_once(':') else {
              return Err(ParseNodeAddressError::MissingPort);
            };

            let port = port.parse().map_err(ParseNodeAddressError::InvalidPort)?;
            let dns = Domain::try_from(domain).map_err(ParseNodeAddressError::InvalidDomain)?;

            Ok(Self {
              kind: Kind::Domain(dns),
              port,
            })
            .map_err(ParseNodeAddressError::InvalidDomain)
          }
        }
      }
    }
  }
}

impl TryFrom<(&str, u16)> for NodeAddress {
  type Error = ParseNodeAddressError;

  fn try_from((domain, port): (&str, u16)) -> Result<Self, Self::Error> {
    let res: Result<IpAddr, _> = domain.parse();
    match res {
      Ok(addr) => Ok(Self {
        kind: Kind::Ip(addr),
        port,
      }),
      Err(_) => Domain::try_from(domain)
        .map(|dns| Self {
          kind: Kind::Domain(dns),
          port,
        })
        .map_err(ParseNodeAddressError::InvalidDomain),
    }
  }
}

impl NodeAddress {
  /// Create a new address from domain and port
  pub fn from_domain_static(s: &'static str, port: u16) -> Result<Self, ParseNodeAddressError> {
    match Domain::new_static(s) {
      Ok(d) => Ok(Self {
        kind: Kind::Domain(d),
        port,
      }),
      Err(e) => Err(ParseNodeAddressError::InvalidDomain(e)),
    }
  }

  /// Returns the domain of the address if this address can only be represented by domain name
  pub fn domain(&self) -> Option<&str> {
    match &self.kind {
      Kind::Ip(_) => None,
      Kind::Domain(name) => Some(name.as_str()),
    }
  }

  /// Returns the ip of the address if this address can be represented by [`IpAddr`]
  pub const fn ip(&self) -> Option<IpAddr> {
    match &self.kind {
      Kind::Ip(addr) => Some(*addr),
      Kind::Domain(_) => None,
    }
  }

  /// Returns the port
  #[inline]
  pub const fn port(&self) -> u16 {
    self.port
  }

  /// Set the port
  #[inline]
  pub fn set_port(&mut self, port: u16) -> &mut Self {
    self.port = port;
    self
  }

  /// Set the port in builder pattern
  #[inline]
  pub const fn with_port(mut self, port: u16) -> Self {
    self.port = port;
    self
  }
}

impl cheap_clone::CheapClone for NodeAddress {}

#[cfg(test)]
mod tests {
  use core::net::{Ipv4Addr, Ipv6Addr};

  use super::*;
  use rand::{distr::Alphanumeric, rng, Rng, RngCore};

  impl NodeAddress {
    fn random_v4_address() -> Self {
      // create a random ipv4 address
      let mut addr = [0u8; 4];
      let mut rng = rng();
      rng.fill_bytes(&mut addr);
      let port = rng.random_range(0..=u16::MAX);

      Self {
        kind: Kind::Ip(IpAddr::V4(Ipv4Addr::from(addr))),
        port,
      }
    }

    fn random_v6_address() -> Self {
      // create a random ipv6 address
      let mut addr = [0u8; 16];
      let mut rng = rng();
      rng.fill_bytes(&mut addr);
      let port = rng.random_range(0..=u16::MAX);

      Self {
        kind: Kind::Ip(IpAddr::V6(Ipv6Addr::from(addr))),
        port,
      }
    }

    fn random_domain_address(size: u8) -> Self {
      // create a random domain address
      let mut trng = rng();

      let domain = rng()
        .sample_iter(Alphanumeric)
        .filter(|c| c.is_ascii_alphabetic())
        .take(size as usize)
        .collect::<Vec<u8>>();
      let domain = String::from_utf8(domain).unwrap();
      let domain = format!("{}.com", domain);
      let port = trng.random_range(0..=u16::MAX);

      Self {
        kind: Kind::Domain(Domain::try_from(domain).unwrap()),
        port,
      }
    }
  }

  #[test]
  fn test_basic() {
    let addr = NodeAddress::from((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080));
    let domain = NodeAddress::try_from(String::from("google.com:8080")).unwrap();
    let domain2 = NodeAddress::try_from(("127.0.0.1", 8080)).unwrap();
    assert!(addr.kind.partial_cmp(&domain2.kind) == Some(core::cmp::Ordering::Equal));
    assert!(addr.cmp(&domain2) == core::cmp::Ordering::Equal);
    println!("{}", addr);
    println!("{}", domain);
    assert!(addr.domain().is_none());
    assert!(addr.ip().is_some());
    assert!(domain.ip().is_none());
    assert!(domain.domain().is_some());
  }

  #[test]
  fn test_ord() {
    let v4 = NodeAddress::random_v4_address();
    let v6 = NodeAddress::random_v6_address();
    let domain = NodeAddress::random_domain_address(32);
    let domain2 = NodeAddress::random_domain_address(63);
    let mut vec = [v4, v6, domain, domain2];
    vec.sort();
    println!("{:?}", vec);

    let v4 = NodeAddress::random_v4_address();
    let v6 = NodeAddress::random_v6_address();
    let domain = NodeAddress::random_domain_address(32);
    let domain2 = NodeAddress::random_domain_address(63);

    let mut v4 = v4.with_port(200);
    assert_eq!(v4.port(), 200);
    v4.set_port(100);
    assert_eq!(v4.port(), 100);
    let mut v6 = v6.with_port(200);
    assert_eq!(v6.port(), 200);
    v6.set_port(100);
    assert_eq!(v6.port(), 100);

    let mut domain = domain.with_port(200);
    assert_eq!(domain.port(), 200);
    domain.set_port(100);
    assert_eq!(domain.port(), 100);
    assert!(domain.ip().is_none());
    assert!(domain.domain().is_some());

    let mut domain2 = domain2.with_port(200);
    assert_eq!(domain2.port(), 200);
    domain2.set_port(100);
    assert_eq!(domain2.port(), 100);
    assert!(domain2.ip().is_none());
    assert!(domain2.domain().is_some());

    let mut vec = [v4, v6, domain, domain2];
    vec.sort();

    let v4 = NodeAddress::random_v4_address();
    let v6 = NodeAddress::random_v6_address();
    let domain = NodeAddress::random_domain_address(32);
    assert!(v4 < domain);
    assert!(v6 < domain);

    assert_eq!(v4.partial_cmp(&domain), Some(core::cmp::Ordering::Less));
  }

  #[cfg(feature = "serde")]
  #[test]
  fn test_serde() {
    let v4 = NodeAddress::random_v4_address();
    let v6 = NodeAddress::random_v6_address();
    let domain = NodeAddress::random_domain_address(63);

    let v4_str = serde_json::to_string(&v4).unwrap();
    let v6_str = serde_json::to_string(&v6).unwrap();
    let domain_str = serde_json::to_string(&domain).unwrap();

    let v4_dec: NodeAddress = serde_json::from_str(&v4_str).unwrap();
    let v6_dec: NodeAddress = serde_json::from_str(&v6_str).unwrap();
    let domain_dec: NodeAddress = serde_json::from_str(&domain_str).unwrap();

    assert_eq!(v4, v4_dec);
    assert_eq!(v6, v6_dec);
    assert_eq!(domain, domain_dec);
  }

  #[test]
  fn test_constructor() {
    let a = NodeAddress::from_domain_static("www.example.com", 80).unwrap();
    assert_eq!(a.domain().unwrap(), "www.example.com");
  }
}
