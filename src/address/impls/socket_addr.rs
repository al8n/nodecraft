use std::net::SocketAddr;

use crate::{Address, Transformable};

#[cfg(feature = "std")]
use crate::utils::invalid_data;

/// The wire error type for [`SocketAddr`].
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum SocketAddrTransformableError {
  /// Returned when the buffer is too small to encode the [`SocketAddr`].
  #[cfg_attr(feature = "std", error(
    "buffer is too small, use `SocketAddr::encoded_len` to pre-allocate a buffer with enough space"
  ))]
  EncodeBufferTooSmall,
  /// Returned when the address family is unknown.
  #[cfg_attr(
    feature = "std",
    error("invalid address family: {0}, only IPv4 and IPv6 are supported")
  )]
  UnknownAddressFamily(u8),
  /// Returned when the address is corrupted.
  #[cfg_attr(feature = "std", error("{0}"))]
  Corrupted(&'static str),
}

impl Address for SocketAddr {}

const MIN_ENCODED_LEN: usize = TAG_SIZE + V4_SIZE + PORT_SIZE;
const V6_ENCODED_LEN: usize = TAG_SIZE + V6_SIZE + PORT_SIZE;
const V6_SIZE: usize = 16;
const V4_SIZE: usize = 4;
const TAG_SIZE: usize = 1;
const PORT_SIZE: usize = core::mem::size_of::<u16>();

#[cfg_attr(all(feature = "async", feature = "std"), async_trait::async_trait)]
impl Transformable for SocketAddr {
  type Error = SocketAddrTransformableError;

  fn encode(&self, dst: &mut [u8]) -> Result<(), Self::Error> {
    let encoded_len = self.encoded_len();
    if dst.len() < encoded_len {
      return Err(Self::Error::EncodeBufferTooSmall);
    }
    dst[0] = match self {
      SocketAddr::V4(_) => 4,
      SocketAddr::V6(_) => 6,
    };
    match self {
      SocketAddr::V4(addr) => {
        dst[1..5].copy_from_slice(&addr.ip().octets());
        dst[5..7].copy_from_slice(&addr.port().to_be_bytes());
      }
      SocketAddr::V6(addr) => {
        dst[1..17].copy_from_slice(&addr.ip().octets());
        dst[17..19].copy_from_slice(&addr.port().to_be_bytes());
      }
    }

    Ok(())
  }

  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn encode_to_writer<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
    match self {
      SocketAddr::V4(addr) => {
        let mut buf = [0u8; 7];
        buf[0] = 4;
        buf[1..5].copy_from_slice(&addr.ip().octets());
        buf[5..7].copy_from_slice(&addr.port().to_be_bytes());
        writer.write_all(&buf)
      }
      SocketAddr::V6(addr) => {
        let mut buf = [0u8; 19];
        buf[0] = 6;
        buf[1..17].copy_from_slice(&addr.ip().octets());
        buf[17..19].copy_from_slice(&addr.port().to_be_bytes());
        writer.write_all(&buf)
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

    match self {
      SocketAddr::V4(addr) => {
        let mut buf = [0u8; 7];
        buf[0] = 4;
        buf[1..5].copy_from_slice(&addr.ip().octets());
        buf[5..7].copy_from_slice(&addr.port().to_be_bytes());
        writer.write_all(&buf).await
      }
      SocketAddr::V6(addr) => {
        let mut buf = [0u8; 19];
        buf[0] = 6;
        buf[1..17].copy_from_slice(&addr.ip().octets());
        buf[17..19].copy_from_slice(&addr.port().to_be_bytes());
        writer.write_all(&buf).await
      }
    }
  }

  fn encoded_len(&self) -> usize {
    1 + match self {
      SocketAddr::V4(_) => 4,
      SocketAddr::V6(_) => 16,
    } + core::mem::size_of::<u16>()
  }

  fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error>
  where
    Self: Sized,
  {
    match src[0] {
      4 => {
        if src.len() < 7 {
          return Err(SocketAddrTransformableError::Corrupted(
            "corrupted socket v4 address",
          ));
        }

        let ip = std::net::Ipv4Addr::new(src[1], src[2], src[3], src[4]);
        let port = u16::from_be_bytes([src[5], src[6]]);
        Ok((MIN_ENCODED_LEN, SocketAddr::from((ip, port))))
      }
      6 => {
        if src.len() < 19 {
          return Err(SocketAddrTransformableError::Corrupted(
            "corrupted socket v6 address",
          ));
        }

        let mut buf = [0u8; 16];
        buf.copy_from_slice(&src[1..17]);
        let ip = std::net::Ipv6Addr::from(buf);
        let port = u16::from_be_bytes([src[17], src[18]]);
        Ok((V6_ENCODED_LEN, SocketAddr::from((ip, port))))
      }
      val => Err(SocketAddrTransformableError::UnknownAddressFamily(val)),
    }
  }

  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn decode_from_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<(usize, Self)>
  where
    Self: Sized,
  {
    use std::net::{Ipv4Addr, Ipv6Addr};

    let mut buf = [0; MIN_ENCODED_LEN];
    reader.read_exact(&mut buf)?;
    match buf[0] {
      4 => {
        let ip = Ipv4Addr::new(buf[1], buf[2], buf[3], buf[4]);
        let port = u16::from_be_bytes([buf[5], buf[6]]);
        Ok((MIN_ENCODED_LEN, SocketAddr::from((ip, port))))
      }
      6 => {
        let mut remaining = [0; V6_ENCODED_LEN - MIN_ENCODED_LEN];
        reader.read_exact(&mut remaining)?;
        let mut ipv6 = [0; V6_SIZE];
        ipv6[..6].copy_from_slice(&buf[1..]);
        ipv6[6..].copy_from_slice(&remaining[..V6_ENCODED_LEN - MIN_ENCODED_LEN - 2]);
        let ip = Ipv6Addr::from(ipv6);
        let port = u16::from_be_bytes([
          remaining[V6_ENCODED_LEN - MIN_ENCODED_LEN - 2],
          remaining[V6_ENCODED_LEN - MIN_ENCODED_LEN - 1],
        ]);
        Ok((V6_ENCODED_LEN, SocketAddr::from((ip, port))))
      }
      val => Err(invalid_data(
        SocketAddrTransformableError::UnknownAddressFamily(val),
      )),
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
    use std::net::{Ipv4Addr, Ipv6Addr};

    let mut buf = [0; MIN_ENCODED_LEN];
    reader.read_exact(&mut buf).await?;
    match buf[0] {
      4 => {
        let ip = Ipv4Addr::new(buf[1], buf[2], buf[3], buf[4]);
        let port = u16::from_be_bytes([buf[5], buf[6]]);
        Ok((MIN_ENCODED_LEN, SocketAddr::from((ip, port))))
      }
      6 => {
        let mut remaining = [0; V6_ENCODED_LEN - MIN_ENCODED_LEN];
        reader.read_exact(&mut remaining).await?;
        let mut ipv6 = [0; V6_SIZE];
        ipv6[..6].copy_from_slice(&buf[1..]);
        ipv6[6..].copy_from_slice(&remaining[..V6_ENCODED_LEN - MIN_ENCODED_LEN - 2]);
        let ip = Ipv6Addr::from(ipv6);
        let port = u16::from_be_bytes([
          remaining[V6_ENCODED_LEN - MIN_ENCODED_LEN - 2],
          remaining[V6_ENCODED_LEN - MIN_ENCODED_LEN - 1],
        ]);
        Ok((V6_ENCODED_LEN, SocketAddr::from((ip, port))))
      }
      val => Err(invalid_data(
        SocketAddrTransformableError::UnknownAddressFamily(val),
      )),
    }
  }
}
