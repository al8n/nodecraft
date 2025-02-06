use std::{
  net::{IpAddr, SocketAddr},
  str::FromStr,
};

mod domain;
pub use domain::{Domain, ParseDomainError};
pub use either::Either;

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

/// An error which can be returned when parsing a [`HostAddr`].
#[derive(Debug, thiserror::Error)]
pub enum ParseHostAddrError {
  /// Returned if the provided str is missing port.
  #[error("address is missing port")]
  PortNotFound,
  /// Returned if the provided str is not a valid address.
  #[error(transparent)]
  Domain(#[from] ParseDomainError),
  /// Returned if the provided str is not a valid port.
  #[error("invalid port: {0}")]
  Port(#[from] core::num::ParseIntError),
}

/// A host address which supports both `domain:port` and socket address.
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
pub struct HostAddr {
  pub(crate) kind: Kind,
  pub(crate) port: u16,
}

impl PartialOrd for HostAddr {
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for HostAddr {
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    match self.kind.cmp(&other.kind) {
      core::cmp::Ordering::Equal => self.port.cmp(&other.port),
      ord => ord,
    }
  }
}

impl core::fmt::Display for HostAddr {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match &self.kind {
      Kind::Ip(addr) => write!(f, "{}", SocketAddr::new(*addr, self.port)),
      Kind::Domain(name) => write!(f, "{}:{}", name.as_str(), self.port),
    }
  }
}

#[cfg(feature = "serde")]
impl serde::Serialize for HostAddr {
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
impl<'de> serde::Deserialize<'de> for HostAddr {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    <&str as serde::Deserialize>::deserialize(deserializer)
      .and_then(|s| Self::from_str(s).map_err(<D::Error as serde::de::Error>::custom))
  }
}

impl From<SocketAddr> for HostAddr {
  fn from(addr: SocketAddr) -> Self {
    Self {
      kind: Kind::Ip(addr.ip()),
      port: addr.port(),
    }
  }
}

impl From<(IpAddr, u16)> for HostAddr {
  fn from(addr: (IpAddr, u16)) -> Self {
    Self {
      kind: Kind::Ip(addr.0),
      port: addr.1,
    }
  }
}

impl From<(Domain, u16)> for HostAddr {
  fn from(addr: (Domain, u16)) -> Self {
    Self {
      kind: Kind::Domain(addr.0),
      port: addr.1,
    }
  }
}

impl TryFrom<String> for HostAddr {
  type Error = ParseHostAddrError;

  fn try_from(s: String) -> Result<Self, Self::Error> {
    Self::from_str(s.as_str())
  }
}

impl TryFrom<&str> for HostAddr {
  type Error = ParseHostAddrError;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    Self::from_str(value)
  }
}

impl FromStr for HostAddr {
  type Err = ParseHostAddrError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let res: Result<SocketAddr, _> = s.parse();
    match res {
      Ok(addr) => Ok(addr.into()),
      Err(_) => {
        let res: Result<IpAddr, _> = s.parse();
        match res {
          Ok(_) => Err(ParseHostAddrError::PortNotFound),
          Err(_) => {
            let Some((domain, port)) = s.rsplit_once(':') else {
              return Err(ParseHostAddrError::PortNotFound);
            };

            let port = port.parse()?;
            let dns = Domain::try_from(domain)?;

            Ok(Self {
              kind: Kind::Domain(dns),
              port,
            })
          }
        }
      }
    }
  }
}

impl TryFrom<(&str, u16)> for HostAddr {
  type Error = ParseHostAddrError;

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
        .map_err(Into::into),
    }
  }
}

impl HostAddr {
  /// Create a new address from domain and port
  pub fn from_domain(s: &str, port: u16) -> Result<Self, ParseHostAddrError> {
    Domain::try_from(s)
      .map(|d| Self {
        kind: Kind::Domain(d),
        port,
      })
      .map_err(Into::into)
  }

  /// Returns the domain of the address if this address can only be represented by domain name
  pub fn domain(&self) -> Option<&str> {
    match &self.kind {
      Kind::Ip(_) => None,
      Kind::Domain(name) => Some(name.as_str()),
    }
  }

  /// Returns the fqdn of the address if this address can only be represented by domain name
  pub fn fqdn(&self) -> Option<&str> {
    match &self.kind {
      Kind::Ip(_) => None,
      Kind::Domain(name) => Some(name.fqdn_str()),
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

  /// Consumes the host addr and returns the inner data
  #[inline]
  pub fn into_inner(self) -> Either<SocketAddr, (u16, Domain)> {
    match self.kind {
      Kind::Ip(addr) => Either::Left(SocketAddr::new(addr, self.port)),
      Kind::Domain(name) => Either::Right((self.port, name)),
    }
  }
}

impl cheap_clone::CheapClone for HostAddr {}

#[cfg(feature = "arbitrary")]
const _: () = {
  use arbitrary::{Arbitrary, Unstructured};

  impl<'a> Arbitrary<'a> for HostAddr {
    fn arbitrary(u: &mut Unstructured<'a>) -> core::result::Result<Self, arbitrary::Error> {
      let kind = u.arbitrary::<u8>()?;
      match kind % 3 {
        0 => Ok(HostAddr::from(SocketAddr::arbitrary(u)?)),
        1 => Ok(HostAddr::from((IpAddr::arbitrary(u)?, u.arbitrary()?))),
        2 => Ok(HostAddr::from((Domain::arbitrary(u)?, u.arbitrary()?))),
        _ => unreachable!(),
      }
    }
  }
};

#[cfg(feature = "quickcheck")]
const _: () = {
  use quickcheck::{Arbitrary, Gen};

  impl Arbitrary for HostAddr {
    fn arbitrary(g: &mut Gen) -> Self {
      let kind = u8::arbitrary(g);
      match kind % 3 {
        0 => HostAddr::from(SocketAddr::arbitrary(g)),
        1 => HostAddr::from((IpAddr::arbitrary(g), u16::arbitrary(g))),
        2 => HostAddr::from((Domain::arbitrary(g), u16::arbitrary(g))),
        _ => unreachable!(),
      }
    }
  }
};

#[cfg(test)]
mod tests {
  use core::net::{Ipv4Addr, Ipv6Addr};

  use super::*;
  use rand::{distr::Alphanumeric, rng, Rng, RngCore};

  impl HostAddr {
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
    let addr = HostAddr::from((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080));
    let domain = HostAddr::try_from(String::from("google.com:8080")).unwrap();
    let domain2 = HostAddr::try_from(("127.0.0.1", 8080)).unwrap();
    assert!(addr.kind.partial_cmp(&domain2.kind) == Some(core::cmp::Ordering::Equal));
    assert!(addr.cmp(&domain2) == core::cmp::Ordering::Equal);
    println!("{}", addr);
    println!("{}", domain);
    assert!(addr.domain().is_none());
    assert!(addr.fqdn().is_none());
    assert!(addr.ip().is_some());
    assert!(domain.ip().is_none());
    assert!(domain.domain().is_some());
  }

  #[test]
  fn test_ord() {
    let v4 = HostAddr::random_v4_address();
    let v6 = HostAddr::random_v6_address();
    let domain = HostAddr::random_domain_address(32);
    let domain2 = HostAddr::random_domain_address(63);
    let mut vec = [v4, v6, domain, domain2];
    vec.sort();
    println!("{:?}", vec);

    let v4 = HostAddr::random_v4_address();
    let v6 = HostAddr::random_v6_address();
    let domain = HostAddr::random_domain_address(32);
    let domain2 = HostAddr::random_domain_address(63);

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

    let v4 = HostAddr::random_v4_address();
    let v6 = HostAddr::random_v6_address();
    let domain = HostAddr::random_domain_address(32);
    assert!(v4 < domain);
    assert!(v6 < domain);

    assert_eq!(v4.partial_cmp(&domain), Some(core::cmp::Ordering::Less));
    assert!(matches!(v4.into_inner(), Either::Left(_)));
  }

  #[cfg(feature = "serde")]
  #[test]
  fn test_serde() {
    let v4 = HostAddr::random_v4_address();
    let v6 = HostAddr::random_v6_address();
    let domain = HostAddr::random_domain_address(63);

    let v4_str = serde_json::to_string(&v4).unwrap();
    let v6_str = serde_json::to_string(&v6).unwrap();
    let domain_str = serde_json::to_string(&domain).unwrap();

    let v4_dec: HostAddr = serde_json::from_str(&v4_str).unwrap();
    let v6_dec: HostAddr = serde_json::from_str(&v6_str).unwrap();
    let domain_dec: HostAddr = serde_json::from_str(&domain_str).unwrap();

    assert_eq!(v4, v4_dec);
    assert_eq!(v6, v6_dec);
    assert_eq!(domain, domain_dec);
  }

  #[test]
  fn test_constructor() {
    let a = HostAddr::from_domain("www.example.com", 80).unwrap();
    assert_eq!(a.domain().unwrap(), "www.example.com");
    assert_eq!(a.port(), 80);
    assert_eq!(a.fqdn().unwrap(), "www.example.com.");

    let a = HostAddr::try_from(("www.example.com", 80)).unwrap();
    assert_eq!(a.domain().unwrap(), "www.example.com");
    assert_eq!(a.port(), 80);
    assert_eq!(a.fqdn().unwrap(), "www.example.com.");
    assert!(matches!(a.into_inner(), Either::Right(_)));
  }

  #[test]
  fn negative_test() {
    let p = HostAddr::try_from("127.0.0.1");
    assert!(matches!(p, Err(ParseHostAddrError::PortNotFound)));

    let p = HostAddr::try_from("www.example.com");
    assert!(matches!(p, Err(ParseHostAddrError::PortNotFound)));
  }
  #[cfg(feature = "serde")]
  #[quickcheck_macros::quickcheck]
  fn fuzzy_serde(node: HostAddr) -> bool {
    let serialized = serde_json::to_string(&node).unwrap();
    let deserialized: HostAddr = serde_json::from_str(&serialized).unwrap();
    node == deserialized
  }
}
