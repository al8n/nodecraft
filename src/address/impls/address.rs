use crate::{NodeAddress, Transformable};

#[cfg(feature = "std")]
use crate::utils::invalid_data;

use std::{
  mem,
  net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
  str::FromStr,
};

use smol_str::SmolStr;

#[derive(PartialEq, Eq, Hash, Clone)]
pub(crate) enum Kind {
  Ip(IpAddr),
  Domain { safe: SmolStr, original: SmolStr },
}

/// An error which can be returned when parsing a [`Address`].
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum ParseAddressError {
  /// Returned if the provided str is missing port.
  #[cfg_attr(feature = "std", error("address is missing port"))]
  MissingPort,
  /// Returned if the provided str is not a valid address.
  #[cfg_attr(feature = "std", error("invalid domain"))]
  InvalidDomain,
  /// Returned if the provided str is not a valid port.
  #[cfg_attr(feature = "std", error("invalid port: {0}"))]
  InvalidPort(#[cfg_attr(feature = "std", from)] std::num::ParseIntError),
}

#[cfg(not(feature = "std"))]
impl core::fmt::Display for ParseAddressError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::MissingPort => write!(f, "address is missing port"),
      Self::InvalidDomain => write!(f, "invalid domain"),
      Self::InvalidPort(port) => write!(f, "invalid port: {port}"),
    }
  }
}

/// An error which can be returned when encoding/decoding a [`Address`].
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum AddressError {
  /// Returned if the provided buffer is too small.
  #[cfg_attr(
    feature = "std",
    error(
      "buffer is too small, use `Address::encoded_len` to pre-allocate a buffer with enough space"
    )
  )]
  EncodeBufferTooSmall,
  /// Returned if fail to parsing the domain address.
  #[cfg_attr(feature = "std", error("{0}"))]
  ParseAddressError(#[cfg_attr(feature = "std", from)] ParseAddressError),
  /// Returned if the provided bytes is corrupted.
  #[cfg_attr(feature = "std", error("{0}"))]
  Corrupted(&'static str),
  /// Returned if the provided bytes contains an unknown address tag.
  #[cfg_attr(feature = "std", error("unknown address tag: {0}"))]
  UnknownAddressTag(u8),
  /// Returned if the provided bytes is not a valid utf8 string.
  #[cfg_attr(feature = "std", error("{0}"))]
  Utf8Error(#[cfg_attr(feature = "std", from)] core::str::Utf8Error),
}

#[cfg(not(feature = "std"))]
impl core::fmt::Display for AddressError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::EncodeBufferTooSmall => write!(f, "buffer is too small, use `Address::encoded_len` to pre-allocate a buffer with enough space"),
      Self::ParseAddressError(err) => write!(f, "{err}"),
      Self::Corrupted(msg) => write!(f, "{msg}"),
      Self::UnknownAddressTag(t) => write!(f, "unknown address tag: {t}"),
      Self::Utf8Error(err) => write!(f, "{err}"),
    }
  }
}

/// A node address which supports both `domain:port` and socket address.
///
/// e.g. Valid format
/// 1. `www.example.com:8080`
/// 2. `[::1]:8080`
/// 3. `127.0.0.1:8080`
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Address {
  pub(crate) kind: Kind,
  pub(crate) port: u16,
}

impl NodeAddress for Address {}

#[cfg(feature = "serde")]
impl serde::Serialize for Address {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    if serializer.is_human_readable() {
      match &self.kind {
        Kind::Ip(ip) => SocketAddr::new(*ip, self.port)
          .to_string()
          .serialize(serializer),
        Kind::Domain { original, .. } => serializer.serialize_str(original.as_str()),
      }
    } else {
      let encoded_len = self.encoded_len();
      let mut buf = Vec::with_capacity(encoded_len);
      self.encode(&mut buf).map_err(serde::ser::Error::custom)?;
      serializer.serialize_bytes(&buf)
    }
  }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Address {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    if deserializer.is_human_readable() {
      <&str as serde::Deserialize>::deserialize(deserializer)
        .and_then(|s| Address::from_str(s).map_err(<D::Error as serde::de::Error>::custom))
    } else {
      <&[u8] as serde::Deserialize>::deserialize(deserializer).and_then(|s| {
        Address::decode(s)
          .map(|(_, b)| b)
          .map_err(<D::Error as serde::de::Error>::custom)
      })
    }
  }
}

impl From<SocketAddr> for Address {
  fn from(addr: SocketAddr) -> Self {
    Self {
      kind: Kind::Ip(addr.ip()),
      port: addr.port(),
    }
  }
}

impl From<(IpAddr, u16)> for Address {
  fn from(addr: (IpAddr, u16)) -> Self {
    Self {
      kind: Kind::Ip(addr.0),
      port: addr.1,
    }
  }
}

impl TryFrom<String> for Address {
  type Error = ParseAddressError;

  fn try_from(s: String) -> Result<Self, Self::Error> {
    Address::from_str(s.as_str())
  }
}

impl TryFrom<&str> for Address {
  type Error = ParseAddressError;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    Address::from_str(value)
  }
}

impl FromStr for Address {
  type Err = ParseAddressError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let res: Result<SocketAddr, _> = s.parse();
    match res {
      Ok(addr) => Ok(addr.into()),
      Err(_) => {
        let res: Result<IpAddr, _> = s.parse();
        match res {
          Ok(_) => Err(ParseAddressError::MissingPort),
          Err(_) => {
            let Some((domain, port)) = s.rsplit_once(':') else {
              return Err(ParseAddressError::MissingPort);
            };

            let port = port.parse().map_err(ParseAddressError::InvalidPort)?;
            idna::domain_to_ascii_strict(domain)
              .map(|mut domain| {
                // make sure we will only issue one query
                if !domain.ends_with('.') {
                  domain.push('.');
                }

                Self {
                  kind: Kind::Domain {
                    safe: SmolStr::from(domain),
                    original: SmolStr::from(s),
                  },
                  port,
                }
              })
              .map_err(|_| ParseAddressError::InvalidDomain)
          }
        }
      }
    }
  }
}

impl TryFrom<(&str, u16)> for Address {
  type Error = ParseAddressError;

  fn try_from((domain, port): (&str, u16)) -> Result<Self, Self::Error> {
    idna::domain_to_ascii_strict(domain)
      .map(|mut domain| {
        // make sure we will only issue one query
        if !domain.ends_with('.') {
          domain.push('.');
        }

        let orig = format!("{domain}:{port}");
        Self {
          kind: Kind::Domain {
            safe: SmolStr::from(domain),
            original: SmolStr::from(orig),
          },
          port,
        }
      })
      .map_err(|_| ParseAddressError::InvalidDomain)
  }
}

impl core::fmt::Display for Address {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self.kind {
      Kind::Ip(addr) => write!(f, "{}", SocketAddr::new(*addr, self.port)),
      Kind::Domain { original, .. } => write!(f, "{original}:{}", self.port),
    }
  }
}

impl Address {
  /// Returns the domain of the address if this address can only be represented by domain name
  pub fn domain(&self) -> Option<&str> {
    match &self.kind {
      Kind::Ip(_) => None,
      Kind::Domain { original, .. } => Some(original.rsplit_once(':').unwrap().1),
    }
  }

  /// Returns the ip of the address if this address can be represented by [`IpAddr`]
  pub const fn ip(&self) -> Option<IpAddr> {
    match &self.kind {
      Kind::Ip(addr) => Some(*addr),
      Kind::Domain { .. } => None,
    }
  }

  /// Returns the port
  pub const fn port(&self) -> u16 {
    self.port
  }

  /// Set the port
  pub fn set_port(&mut self, port: u16) {
    self.port = port;
  }

  /// Set the port in builder pattern
  pub fn with_port(mut self, port: u16) -> Self {
    self.port = port;
    self
  }
}

const PORT_SIZE: usize = mem::size_of::<u16>();
const TAG_SIZE: usize = 1;
/// A domain is less than 255 bytes, so u8 is enough to represent the length of a domain.
const DOMAIN_LEN_SIZE: usize = 1;
const V6_SIZE: usize = 16;
const V4_SIZE: usize = 4;
/// The minimum encoded length of an address.
///
/// TAG_SIZE + DOMAIN_LEN_SIZE + MINIMUM_DOMAIN_LEN + PORT_SIZE = 1 + 1 + 4 + 2 = 8 for domain
/// so SocketAddrV4 is the minimum encoded length
const MIN_ENCODED_LEN: usize = TAG_SIZE + V4_SIZE + PORT_SIZE;
const V6_ENCODED_LEN: usize = TAG_SIZE + V6_SIZE + PORT_SIZE;

#[cfg_attr(all(feature = "async", feature = "std"), async_trait::async_trait)]
impl Transformable for Address {
  type Error = AddressError;

  fn encode(&self, dst: &mut [u8]) -> Result<(), Self::Error> {
    if dst.len() < self.encoded_len() {
      return Err(AddressError::EncodeBufferTooSmall);
    }

    match &self.kind {
      Kind::Ip(addr) => match addr {
        IpAddr::V4(addr) => {
          dst[0] = 4;
          dst[1..5].copy_from_slice(&addr.octets());
          dst[5..7].copy_from_slice(&self.port.to_be_bytes());
        }
        IpAddr::V6(addr) => {
          dst[0] = 6;
          dst[1..17].copy_from_slice(&addr.octets());
          dst[17..19].copy_from_slice(&self.port.to_be_bytes());
        }
      },
      Kind::Domain { safe, .. } => {
        let mut cur = 0;
        dst[cur] = 0;
        cur += TAG_SIZE;
        dst[cur] = safe.len() as u8;
        cur += DOMAIN_LEN_SIZE;
        dst[cur..cur + safe.len()].copy_from_slice(safe.as_bytes());
        cur += safe.len();
        dst[cur..cur + PORT_SIZE].copy_from_slice(&self.port.to_be_bytes());
      }
    }
    Ok(())
  }

  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn encode_to_writer<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
    match &self.kind {
      Kind::Ip(addr) => match addr {
        IpAddr::V4(addr) => {
          let mut dst = [0; 7];
          dst[0] = 4;
          dst[1..5].copy_from_slice(&addr.octets());
          dst[5..7].copy_from_slice(&self.port.to_be_bytes());
          writer.write_all(&dst)
        }
        IpAddr::V6(addr) => {
          let mut dst = [0; 19];
          dst[0] = 6;
          dst[1..17].copy_from_slice(&addr.octets());
          dst[17..19].copy_from_slice(&self.port.to_be_bytes());
          writer.write_all(&dst)
        }
      },
      Kind::Domain { safe, .. } => {
        let mut dst = vec![0; self.encoded_len()];
        let mut cur = 0;
        dst[cur] = 0;
        cur += TAG_SIZE;
        dst[cur] = safe.len() as u8;
        cur += DOMAIN_LEN_SIZE;
        dst[cur..cur + safe.len()].copy_from_slice(safe.as_bytes());
        cur += safe.len();
        dst[cur..cur + PORT_SIZE].copy_from_slice(&self.port.to_be_bytes());
        writer.write_all(&dst)
      }
    }
  }

  #[cfg(all(feature = "async", feature = "std"))]
  #[cfg_attr(docsrs, doc(cfg(all(feature = "async", feature = "std"))))]
  async fn encode_to_async_writer<W: futures::io::AsyncWrite + Send + Unpin>(
    &self,
    writer: &mut W,
  ) -> std::io::Result<()> {
    use futures::AsyncWriteExt;

    match &self.kind {
      Kind::Ip(addr) => match addr {
        IpAddr::V4(addr) => {
          let mut dst = [0; 7];
          dst[0] = 4;
          dst[1..5].copy_from_slice(&addr.octets());
          dst[5..7].copy_from_slice(&self.port.to_be_bytes());
          writer.write_all(&dst).await
        }
        IpAddr::V6(addr) => {
          let mut dst = [0; 19];
          dst[0] = 6;
          dst[1..17].copy_from_slice(&addr.octets());
          dst[17..19].copy_from_slice(&self.port.to_be_bytes());
          writer.write_all(&dst).await
        }
      },
      Kind::Domain { safe, .. } => {
        let mut dst = vec![0; self.encoded_len()];
        let mut cur = 0;
        dst[cur] = 0;
        cur += TAG_SIZE;
        dst[cur] = safe.len() as u8;
        cur += DOMAIN_LEN_SIZE;
        dst[cur..cur + safe.len()].copy_from_slice(safe.as_bytes());
        cur += safe.len();
        dst[cur..cur + PORT_SIZE].copy_from_slice(&self.port.to_be_bytes());
        writer.write_all(&dst).await
      }
    }
  }

  fn encoded_len(&self) -> usize {
    match &self.kind {
      Kind::Ip(addr) => match addr {
        IpAddr::V4(_) => TAG_SIZE + V4_SIZE + PORT_SIZE,
        IpAddr::V6(_) => TAG_SIZE + V6_SIZE + PORT_SIZE,
      },
      Kind::Domain { safe, .. } => TAG_SIZE + DOMAIN_LEN_SIZE + safe.len() + PORT_SIZE,
    }
  }

  fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error>
  where
    Self: Sized,
  {
    if src.len() < TAG_SIZE + DOMAIN_LEN_SIZE {
      return Err(AddressError::Corrupted("corrupted address"));
    }

    let mut cur = 0;
    let tag = src[0];
    cur += TAG_SIZE;

    match tag {
      0 => {
        let len = src[cur] as usize;
        cur += DOMAIN_LEN_SIZE;
        if src.len() < cur + len + PORT_SIZE {
          return Err(AddressError::Corrupted("corrupted address"));
        }

        let s = core::str::from_utf8(&src[cur..cur + len])?;
        cur += len;
        let port = u16::from_be_bytes([src[cur], src[cur + 1]]);
        cur += 2;
        let original = format!("{s}:{port}");
        Address::from_str(original.as_str())
          .map(|addr| (cur, addr))
          .map_err(Into::into)
      }
      4 => {
        if src.len() < cur + V4_SIZE + PORT_SIZE {
          return Err(AddressError::Corrupted("corrupted address"));
        }

        let ip = Ipv4Addr::new(src[cur], src[cur + 1], src[cur + 2], src[cur + 3]);
        let port = u16::from_be_bytes([src[cur + V4_SIZE], src[cur + V4_SIZE + 1]]);
        Ok((MIN_ENCODED_LEN, SocketAddr::from((ip, port)).into()))
      }
      6 => {
        if src.len() < cur + V6_SIZE + PORT_SIZE {
          return Err(AddressError::Corrupted("corrupted address"));
        }

        let mut buf = [0u8; V6_SIZE];
        buf.copy_from_slice(&src[cur..cur + V6_SIZE]);
        let ip = Ipv6Addr::from(buf);
        let port = u16::from_be_bytes([src[cur + V6_SIZE], src[cur + V6_SIZE + 1]]);
        Ok((V6_ENCODED_LEN, SocketAddr::from((ip, port)).into()))
      }
      val => Err(AddressError::UnknownAddressTag(val)),
    }
  }

  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn decode_from_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<(usize, Self)>
  where
    Self: Sized,
  {
    let mut buf = [0u8; MIN_ENCODED_LEN];
    reader.read_exact(&mut buf)?;
    match buf[0] {
      0 => {
        const INLINE: usize = 32;
        const READED: usize = 5;

        let len = buf[1] as usize;
        let remaining = len + PORT_SIZE - READED;
        let addr_len = remaining + READED;
        if addr_len < INLINE {
          let mut domain = [0; INLINE];
          domain[..READED].copy_from_slice(&buf[2..]);
          reader.read_exact(&mut domain[READED..READED + remaining])?;
          let src = core::str::from_utf8(&domain).map_err(invalid_data)?;
          Address::from_str(src).map_err(invalid_data)
        } else {
          let mut addr = vec![0; addr_len];
          addr[..READED].copy_from_slice(&buf[2..]);
          reader.read_exact(&mut addr[READED..])?;
          let src = core::str::from_utf8(&addr).map_err(invalid_data)?;
          Address::from_str(src).map_err(invalid_data)
        }
        .map(|a| (MIN_ENCODED_LEN + remaining, a))
      }
      4 => Ok((
        MIN_ENCODED_LEN,
        Self {
          kind: Kind::Ip(IpAddr::V4(Ipv4Addr::new(buf[1], buf[2], buf[3], buf[4]))),
          port: u16::from_be_bytes([buf[5], buf[6]]),
        },
      )),
      6 => {
        let mut remaining = [0u8; V6_ENCODED_LEN - MIN_ENCODED_LEN];
        reader.read_exact(&mut remaining)?;
        let mut ipv6 = [0; V6_SIZE];
        ipv6[..6].copy_from_slice(&buf[1..]);
        ipv6[6..].copy_from_slice(&remaining[..V6_ENCODED_LEN - MIN_ENCODED_LEN - 2]);
        Ok((
          V6_ENCODED_LEN,
          Self {
            kind: Kind::Ip(IpAddr::V6(Ipv6Addr::from(ipv6))),
            port: u16::from_be_bytes([
              remaining[V6_ENCODED_LEN - MIN_ENCODED_LEN - 2],
              remaining[V6_ENCODED_LEN - MIN_ENCODED_LEN - 1],
            ]),
          },
        ))
      }
      t => Err(invalid_data(AddressError::UnknownAddressTag(t))),
    }
  }

  #[cfg(all(feature = "async", feature = "std"))]
  #[cfg_attr(docsrs, doc(cfg(all(feature = "async", feature = "std"))))]
  async fn decode_from_async_reader<R: futures::io::AsyncRead + Send + Unpin>(
    reader: &mut R,
  ) -> std::io::Result<(usize, Self)>
  where
    Self: Sized,
  {
    use futures::AsyncReadExt;

    let mut buf = [0u8; MIN_ENCODED_LEN];
    reader.read_exact(&mut buf).await?;
    match buf[0] {
      0 => {
        const INLINE: usize = 32;
        const READED: usize = 5;
        let len = buf[1] as usize;
        let remaining = len + PORT_SIZE - READED;
        let addr_len = remaining + READED;
        if addr_len < INLINE {
          let mut domain = [0; INLINE];
          domain[..READED].copy_from_slice(&buf[2..]);
          reader
            .read_exact(&mut domain[READED..READED + remaining])
            .await?;
          let src = core::str::from_utf8(&domain).map_err(invalid_data)?;
          Address::from_str(src).map_err(invalid_data)
        } else {
          let mut addr = vec![0; addr_len];
          addr[..READED].copy_from_slice(&buf[2..]);
          reader.read_exact(&mut addr[READED..]).await?;
          let src = core::str::from_utf8(&addr).map_err(invalid_data)?;
          Address::from_str(src).map_err(invalid_data)
        }
        .map(|a| (MIN_ENCODED_LEN + remaining, a))
      }
      4 => Ok((
        MIN_ENCODED_LEN,
        Self {
          kind: Kind::Ip(IpAddr::V4(Ipv4Addr::new(buf[1], buf[2], buf[3], buf[4]))),
          port: u16::from_be_bytes([buf[5], buf[6]]),
        },
      )),
      6 => {
        let mut remaining = [0u8; V6_ENCODED_LEN - MIN_ENCODED_LEN];
        reader.read_exact(&mut remaining).await?;
        let mut ipv6 = [0; V6_SIZE];
        ipv6[..6].copy_from_slice(&buf[1..]);
        ipv6[6..].copy_from_slice(&remaining[..V6_ENCODED_LEN - MIN_ENCODED_LEN - 2]);
        Ok((
          V6_ENCODED_LEN,
          Self {
            kind: Kind::Ip(IpAddr::V6(Ipv6Addr::from(ipv6))),
            port: u16::from_be_bytes([
              remaining[V6_ENCODED_LEN - MIN_ENCODED_LEN - 2],
              remaining[V6_ENCODED_LEN - MIN_ENCODED_LEN - 1],
            ]),
          },
        ))
      }
      t => Err(invalid_data(AddressError::UnknownAddressTag(t))),
    }
  }
}
