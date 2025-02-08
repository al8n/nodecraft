use core::net::{IpAddr, SocketAddr};
pub use either::Either;

use super::{DomainRef, ParseHostAddrError};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Repr<'a> {
  Ip(IpAddr),
  DomainRef(DomainRef<'a>),
}

impl PartialOrd for Repr<'_> {
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Repr<'_> {
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    match (self, other) {
      (Self::Ip(a), Self::Ip(b)) => a.cmp(b),
      (Self::DomainRef(a), Self::DomainRef(b)) => a.cmp(b),
      (Self::Ip(_), Self::DomainRef(_)) => core::cmp::Ordering::Less,
      (Self::DomainRef(_), Self::Ip(_)) => core::cmp::Ordering::Greater,
    }
  }
}

/// A host address which supports both `domain:port` and socket address.
///
/// e.g. Valid format
/// 1. `www.example.com:8080`
/// 2. `[::1]:8080`
/// 3. `127.0.0.1:8080`
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct HostAddrRef<'a> {
  kind: Repr<'a>,
  port: u16,
}

impl PartialOrd for HostAddrRef<'_> {
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for HostAddrRef<'_> {
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    match self.kind.cmp(&other.kind) {
      core::cmp::Ordering::Equal => self.port.cmp(&other.port),
      ord => ord,
    }
  }
}

impl core::fmt::Display for HostAddrRef<'_> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match &self.kind {
      Repr::Ip(addr) => write!(f, "{}", SocketAddr::new(*addr, self.port)),
      Repr::DomainRef(name) => write!(f, "{}:{}", name.as_str(), self.port),
    }
  }
}

#[cfg(feature = "serde")]
impl serde::Serialize for HostAddrRef<'_> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    use smol_str03::ToSmolStr;
    match &self.kind {
      Repr::Ip(ip) => SocketAddr::new(*ip, self.port)
        .to_smolstr()
        .serialize(serializer),
      Repr::DomainRef(name) => {
        let s = format!("{}:{}", name.as_str(), self.port);
        s.serialize(serializer)
      }
    }
  }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for HostAddrRef<'de> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    <&str as serde::Deserialize>::deserialize(deserializer)
      .and_then(|s| Self::try_from(s).map_err(<D::Error as serde::de::Error>::custom))
  }
}

impl From<SocketAddr> for HostAddrRef<'_> {
  fn from(addr: SocketAddr) -> Self {
    Self {
      kind: Repr::Ip(addr.ip()),
      port: addr.port(),
    }
  }
}

impl From<(IpAddr, u16)> for HostAddrRef<'_> {
  fn from(addr: (IpAddr, u16)) -> Self {
    Self {
      kind: Repr::Ip(addr.0),
      port: addr.1,
    }
  }
}

impl<'a> From<(DomainRef<'a>, u16)> for HostAddrRef<'a> {
  fn from(addr: (DomainRef<'a>, u16)) -> Self {
    Self {
      kind: Repr::DomainRef(addr.0),
      port: addr.1,
    }
  }
}

impl<'a> TryFrom<&'a str> for HostAddrRef<'a> {
  type Error = ParseHostAddrError;

  fn try_from(s: &'a str) -> Result<Self, Self::Error> {
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
            let dns = DomainRef::try_from(domain)?;

            Ok(Self {
              kind: Repr::DomainRef(dns),
              port,
            })
          }
        }
      }
    }
  }
}

impl<'a> TryFrom<(&'a str, u16)> for HostAddrRef<'a> {
  type Error = ParseHostAddrError;

  fn try_from((domain, port): (&'a str, u16)) -> Result<Self, Self::Error> {
    let res: Result<IpAddr, _> = domain.parse();
    match res {
      Ok(addr) => Ok(Self {
        kind: Repr::Ip(addr),
        port,
      }),
      Err(_) => DomainRef::try_from(domain)
        .map(|dns| Self {
          kind: Repr::DomainRef(dns),
          port,
        })
        .map_err(Into::into),
    }
  }
}

impl<'a> HostAddrRef<'a> {
  /// Create a new address from domain and port
  pub fn from_domain(s: &'a str, port: u16) -> Result<Self, ParseHostAddrError> {
    DomainRef::try_from(s)
      .map(|d| Self {
        kind: Repr::DomainRef(d),
        port,
      })
      .map_err(Into::into)
  }

  /// Returns the domain of the address if this address can only be represented by domain name
  pub fn domain(&self) -> Option<&DomainRef<'a>> {
    match &self.kind {
      Repr::Ip(_) => None,
      Repr::DomainRef(name) => Some(name),
    }
  }

  /// Returns the ip of the address if this address can be represented by [`IpAddr`]
  pub const fn ip(&self) -> Option<IpAddr> {
    match &self.kind {
      Repr::Ip(addr) => Some(*addr),
      Repr::DomainRef(_) => None,
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
  pub fn into_inner(self) -> Either<SocketAddr, (u16, DomainRef<'a>)> {
    match self.kind {
      Repr::Ip(addr) => Either::Left(SocketAddr::new(addr, self.port)),
      Repr::DomainRef(name) => Either::Right((self.port, name)),
    }
  }
}

#[cfg(all(any(feature = "std", feature = "alloc"), test))]
mod tests {
  use core::net::Ipv4Addr;

  use super::*;

  #[test]
  fn test_basic() {
    let addr = HostAddrRef::from((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080));
    let domain = HostAddrRef::try_from("google.com:8080").unwrap();
    let domain2 = HostAddrRef::try_from(("127.0.0.1", 8080)).unwrap();
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
  fn test_constructor() {
    let a = HostAddrRef::from_domain("www.example.com", 80).unwrap();
    assert_eq!(a.domain().unwrap().as_str(), "www.example.com");
    assert_eq!(a.port(), 80);

    let a = HostAddrRef::try_from(("www.example.com.", 80)).unwrap();
    assert_eq!(a.domain().unwrap().as_str(), "www.example.com");
    assert_eq!(a.port(), 80);
    assert!(matches!(a.into_inner(), Either::Right(_)));
  }

  #[test]
  fn negative_test() {
    let p = HostAddrRef::try_from("127.0.0.1");
    assert!(matches!(p, Err(ParseHostAddrError::PortNotFound)));

    let p = HostAddrRef::try_from("www.example.com");
    assert!(matches!(p, Err(ParseHostAddrError::PortNotFound)));
  }
}
