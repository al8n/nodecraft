use core::{borrow::Borrow, mem};

use byteorder::{ByteOrder, NetworkEndian};
use smol_str03::SmolStr;

use crate::{Id, Transformable};

#[cfg(feature = "std")]
use crate::utils::invalid_data;

#[cfg(not(all(feature = "std", feature = "alloc")))]
use ::alloc::{string::String, vec::Vec};

/// Errors that can occur when transforming an [`NodeId`].
#[derive(Debug, thiserror::Error)]
pub enum NodeIdTransformError {
  /// Returned when the id is empty.
  #[error("id cannot be empty")]
  Empty,
  /// Returned when the id is too large.
  #[error("id is too large, maximum size is 512 bytes, but got {0} bytes")]
  TooLarge(usize),
  /// Returned when the buffer is too small to encode the [`Id`].
  #[error("buffer is too small, use Id::encoded_size to pre-allocate a buffer with enough space")]
  EncodeBufferTooSmall,
  /// Returned when the id is corrupted.
  #[error("corrupted id")]
  Corrupted,
  /// Returned when the id is not a valid utf8 string.
  #[error(transparent)]
  Utf8Error(#[from] core::str::Utf8Error),
}

/// A unique string identifying a server for all time.
/// The maximum length of an id is 512 bytes.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(
  feature = "rkyv",
  derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(
  feature = "rkyv",
  rkyv(compare(PartialEq), derive(PartialEq, Eq, PartialOrd, Ord, Hash),)
)]
pub struct NodeId(SmolStr);

impl Id for NodeId {}

impl NodeId {
  /// The maximum length of an id is 512 bytes.
  pub const MAX_SIZE: usize = 512;

  /// Creates a new `Id` from the source.
  pub fn new<T: AsRef<str>>(src: T) -> Result<Self, NodeIdTransformError> {
    let src = src.as_ref();
    if src.is_empty() {
      return Err(NodeIdTransformError::Empty);
    }

    if src.len() > Self::MAX_SIZE {
      return Err(NodeIdTransformError::TooLarge(src.len()));
    }

    Ok(Self(SmolStr::new(src)))
  }

  /// converts the `Id` into a `&str`.
  pub fn as_str(&self) -> &str {
    self.0.as_ref()
  }

  /// Returns a byte slice.
  /// To convert the byte slice back into a string slice, use the [`core::str::from_utf8`] function.
  pub fn as_bytes(&self) -> &[u8] {
    self.0.as_bytes()
  }
}

#[cfg(feature = "std")]
const INLINE: usize = 64;
const LENGTH_SIZE: usize = mem::size_of::<u16>();

#[cfg(feature = "transformable")]
impl Transformable for NodeId {
  type Error = NodeIdTransformError;

  fn encode(&self, dst: &mut [u8]) -> Result<usize, Self::Error> {
    let encoded_len = self.encoded_len();
    if dst.len() < encoded_len {
      return Err(NodeIdTransformError::EncodeBufferTooSmall);
    }

    let mut cur = 0;
    NetworkEndian::write_u16(&mut dst[..LENGTH_SIZE], self.0.len() as u16);
    cur += LENGTH_SIZE;
    dst[cur..cur + self.0.len()].copy_from_slice(self.0.as_bytes());
    Ok(encoded_len)
  }

  /// Encodes the value into the given writer.
  ///
  /// # Note
  /// The implementation of this method is not optimized, which means
  /// if your writer is expensive (e.g. [`TcpStream`](std::net::TcpStream), [`File`](std::fs::File)),
  /// it is better to use a [`BufWriter`](std::io::BufWriter)
  /// to wrap your orginal writer to cut down the number of I/O times.
  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn encode_to_writer<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
    let encoded_len = self.encoded_len();
    let len = self.0.len() as u16;
    let mut len_buf = [0; core::mem::size_of::<u16>()];
    NetworkEndian::write_u16(&mut len_buf, len);
    writer.write_all(&len_buf).and_then(|_| {
      writer
        .write_all(self.0.as_str().as_bytes())
        .map(|_| encoded_len)
    })
  }

  /// Encodes the value into the given async writer.
  ///
  /// # Note
  /// The implementation of this method is not optimized, which means
  /// if your writer is expensive (e.g. `TcpStream`, `File`),
  /// it is better to use a [`BufWriter`](futures::io::BufWriter)
  /// to wrap your orginal writer to cut down the number of I/O times.
  #[cfg(feature = "async")]
  #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
  async fn encode_to_async_writer<W: futures::io::AsyncWrite + Send + Unpin>(
    &self,
    writer: &mut W,
  ) -> std::io::Result<usize>
  where
    Self::Error: Send + Sync + 'static,
  {
    use futures::AsyncWriteExt;

    let encoded_len = self.encoded_len();
    let len = self.0.len() as u16;
    let mut len_buf = [0; core::mem::size_of::<u16>()];
    NetworkEndian::write_u16(&mut len_buf, len);
    writer.write_all(&len_buf).await?;
    writer
      .write_all(self.0.as_str().as_bytes())
      .await
      .map(|_| encoded_len)
  }

  fn encoded_len(&self) -> usize {
    LENGTH_SIZE + self.0.len()
  }

  fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error>
  where
    Self: Sized,
  {
    if src.len() < LENGTH_SIZE {
      return Err(NodeIdTransformError::Corrupted);
    }

    let len = NetworkEndian::read_u16(&src[..core::mem::size_of::<u16>()]) as usize;
    if src.len() < LENGTH_SIZE + len {
      return Err(NodeIdTransformError::Corrupted);
    }

    let id = Self::new(core::str::from_utf8(&src[LENGTH_SIZE..LENGTH_SIZE + len])?)?;
    Ok((LENGTH_SIZE + len, id))
  }

  /// Decodes the value from the given reader.
  ///
  /// # Note
  /// The implementation of this method is not optimized, which means
  /// if your reader is expensive (e.g. [`TcpStream`](std::net::TcpStream), [`File`](std::fs::File)),
  /// it is better to use a [`BufReader`](std::io::BufReader)
  /// to wrap your orginal reader to cut down the number of I/O times.
  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn decode_from_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<(usize, Self)>
  where
    Self: Sized,
  {
    let mut len_buf = [0; core::mem::size_of::<u16>()];
    reader.read_exact(&mut len_buf)?;
    let len = NetworkEndian::read_u16(&len_buf) as usize;
    if len == 0 {
      return Err(invalid_data(NodeIdTransformError::Empty));
    }

    if len > Self::MAX_SIZE {
      return Err(invalid_data(NodeIdTransformError::TooLarge(len)));
    }

    if len < INLINE {
      let mut buf = [0; INLINE];
      reader.read_exact(&mut buf[..len])?;
      core::str::from_utf8(&buf[..len])
        .map(|s| (LENGTH_SIZE + len, Self(SmolStr::new(s))))
        .map_err(invalid_data)
    } else {
      let mut buf = vec![0; len];
      reader.read_exact(&mut buf)?;
      core::str::from_utf8(&buf)
        .map(|s| (LENGTH_SIZE + len, Self(SmolStr::new(s))))
        .map_err(invalid_data)
    }
  }

  /// Decodes the value from the given async reader.
  ///
  /// # Note
  /// The implementation of this method is not optimized, which means
  /// if your reader is expensive (e.g. `TcpStream`, `File`),
  /// it is better to use a [`BufReader`](futures::io::BufReader)
  /// to wrap your orginal reader to cut down the number of I/O times.
  #[cfg(feature = "async")]
  #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
  async fn decode_from_async_reader<R: futures::io::AsyncRead + Send + Unpin>(
    reader: &mut R,
  ) -> std::io::Result<(usize, Self)>
  where
    Self: Sized,
    Self::Error: Send + Sync + 'static,
  {
    use futures::AsyncReadExt;

    let mut len_buf = [0; core::mem::size_of::<u16>()];
    reader.read_exact(&mut len_buf).await?;
    let len = NetworkEndian::read_u16(&len_buf) as usize;
    if len == 0 {
      return Err(invalid_data(NodeIdTransformError::Empty));
    }

    if len > Self::MAX_SIZE {
      return Err(invalid_data(NodeIdTransformError::TooLarge(len)));
    }

    if len < INLINE {
      let mut buf = [0; INLINE];
      reader.read_exact(&mut buf[..len]).await?;
      core::str::from_utf8(&buf[..len])
        .map(|s| (LENGTH_SIZE + len, Self(SmolStr::new(s))))
        .map_err(invalid_data)
    } else {
      let mut buf = vec![0; len];
      reader.read_exact(&mut buf).await?;
      core::str::from_utf8(&buf)
        .map(|s| (LENGTH_SIZE + len, Self(SmolStr::new(s))))
        .map_err(invalid_data)
    }
  }
}

impl core::str::FromStr for NodeId {
  type Err = NodeIdTransformError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::new(s)
  }
}

impl Borrow<str> for NodeId {
  fn borrow(&self) -> &str {
    self.as_str()
  }
}

impl AsRef<str> for NodeId {
  fn as_ref(&self) -> &str {
    self.0.as_ref()
  }
}

impl core::fmt::Display for NodeId {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.0.fmt(f)
  }
}

impl core::fmt::Debug for NodeId {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.0.fmt(f)
  }
}

impl cheap_clone::CheapClone for NodeId {}

impl From<NodeId> for SmolStr {
  fn from(id: NodeId) -> Self {
    id.0
  }
}

impl TryFrom<&[u8]> for NodeId {
  type Error = NodeIdTransformError;

  fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
    Self::new(core::str::from_utf8(value)?)
  }
}

#[cfg(feature = "alloc")]
impl TryFrom<Vec<u8>> for NodeId {
  type Error = NodeIdTransformError;

  fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
    let s =
      String::from_utf8(value).map_err(|e| NodeIdTransformError::Utf8Error(e.utf8_error()))?;

    if s.len() > Self::MAX_SIZE {
      return Err(NodeIdTransformError::TooLarge(s.len()));
    }

    if s.is_empty() {
      return Err(NodeIdTransformError::Empty);
    }

    Ok(Self(s.into()))
  }
}

#[cfg(feature = "alloc")]
impl TryFrom<String> for NodeId {
  type Error = NodeIdTransformError;

  fn try_from(s: String) -> Result<Self, Self::Error> {
    if s.len() > Self::MAX_SIZE {
      return Err(NodeIdTransformError::TooLarge(s.len()));
    }

    if s.is_empty() {
      return Err(NodeIdTransformError::Empty);
    }

    Ok(Self(s.into()))
  }
}

#[cfg(test)]
mod tests {
  use core::str::FromStr;

  use rand::{distr::Alphanumeric, rng};

  use super::*;

  impl NodeId {
    fn random(size: usize) -> Self {
      use rand::Rng;
      let id = rng()
        .sample_iter(Alphanumeric)
        .take(size)
        .collect::<Vec<u8>>();

      NodeId::try_from(id).unwrap()
    }
  }

  #[test]
  fn test_basic() {
    let id = NodeId::try_from(b"test".as_slice()).unwrap();
    assert_eq!(id.as_str(), "test");
    assert_eq!(id.as_ref(), "test");
    assert_eq!(id.as_bytes(), b"test");
    println!("{id}");
    println!("{id:?}");

    let _id = NodeId::from_str("test1").unwrap();

    assert!(NodeId::new("").is_err());
    assert!(NodeId::new("a".repeat(513)).is_err());
  }

  #[test]
  #[cfg(feature = "alloc")]
  fn test_try_from() {
    let id = NodeId::try_from(String::from("test")).unwrap();
    assert_eq!(id.as_str(), "test");
    assert_eq!(id.as_ref(), "test");
    assert!(NodeId::try_from(String::new()).is_err());
    assert!(NodeId::try_from("a".repeat(513)).is_err());

    let id = NodeId::try_from(Vec::from("test".as_bytes())).unwrap();

    assert_eq!(id.as_str(), "test");
    assert_eq!(id.as_ref(), "test");
    assert!(NodeId::try_from(Vec::new()).is_err());
    assert!(NodeId::try_from("a".repeat(513).into_bytes()).is_err());

    let id = SmolStr::from(id);
    assert_eq!(id.as_str(), "test");
  }

  #[test]
  #[cfg(feature = "std")]
  fn test_borrow() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    let id = NodeId::try_from(b"test".as_slice()).unwrap();
    set.insert(id.clone());
    assert!(set.contains("test"));
  }

  #[cfg(feature = "transformable")]
  #[test]
  fn test_transformable() {
    let id = NodeId::random(32);
    let mut buf = vec![0; id.encoded_len()];
    id.encode(&mut buf).unwrap();
    assert!(id.encode(&mut []).is_err());
    let (size, decoded) = NodeId::decode(&buf).unwrap();
    assert!(NodeId::decode(&[]).is_err());
    assert!(NodeId::decode(&[0, 1]).is_err());
    assert_eq!(size, id.encoded_len());
    assert_eq!(id, decoded);

    let id = NodeId::random(96);
    let mut buf = vec![0; id.encoded_len()];
    id.encode(&mut buf).unwrap();
    let (size, decoded) = NodeId::decode(&buf).unwrap();
    assert_eq!(size, id.encoded_len());
    assert_eq!(id, decoded);
  }

  #[cfg(feature = "transformable")]
  #[test]
  fn test_transformable_io() {
    use std::io::Cursor;

    let id = NodeId::random(32);
    let mut buf = Vec::new();
    id.encode_to_writer(&mut buf).unwrap();
    let mut buf = Cursor::new(buf);
    let (size, decoded) = NodeId::decode_from_reader(&mut buf).unwrap();
    assert_eq!(size, id.encoded_len());
    assert_eq!(id, decoded);

    let id = NodeId::random(96);
    let mut buf = Vec::new();
    id.encode_to_writer(&mut buf).unwrap();
    let mut buf = Cursor::new(buf);
    assert!(NodeId::decode_from_reader(&mut Cursor::new(&[])).is_err());
    assert!(NodeId::decode_from_reader(&mut Cursor::new(&[255, 255])).is_err());
    let (size, decoded) = NodeId::decode_from_reader(&mut buf).unwrap();
    assert_eq!(size, id.encoded_len());
    assert_eq!(id, decoded);

    let id = NodeId::random(32);
    let mut buf = vec![0; id.encoded_len()];
    id.encode(&mut buf).unwrap();
    let mut buf = Cursor::new(buf);
    let (size, decoded) = NodeId::decode_from_reader(&mut buf).unwrap();
    assert_eq!(size, id.encoded_len());
    assert_eq!(id, decoded);

    let id = NodeId::random(96);
    let mut buf = vec![0; id.encoded_len()];
    id.encode(&mut buf).unwrap();
    let mut buf = Cursor::new(buf);
    let (size, decoded) = NodeId::decode_from_reader(&mut buf).unwrap();
    assert_eq!(size, id.encoded_len());
    assert_eq!(id, decoded);

    let id = NodeId::random(32);
    let mut buf = Vec::new();
    id.encode_to_writer(&mut buf).unwrap();
    let (size, decoded) = NodeId::decode(&buf).unwrap();
    assert_eq!(size, id.encoded_len());
    assert_eq!(id, decoded);

    let id = NodeId::random(96);
    let mut buf = Vec::new();
    id.encode_to_writer(&mut buf).unwrap();
    let (size, decoded) = NodeId::decode(&buf).unwrap();
    assert_eq!(size, id.encoded_len());
    assert_eq!(id, decoded);
  }

  #[cfg(all(feature = "async", feature = "transformable"))]
  #[tokio::test]
  async fn test_transformable_async_io() {
    use futures::io::Cursor;

    let id = NodeId::random(32);
    let mut buf = Vec::new();
    id.encode_to_async_writer(&mut buf).await.unwrap();
    let mut buf = Cursor::new(buf);
    let (size, decoded) = NodeId::decode_from_async_reader(&mut buf).await.unwrap();
    assert_eq!(size, id.encoded_len());
    assert_eq!(id, decoded);

    let id = NodeId::random(96);
    let mut buf = Vec::new();
    id.encode_to_async_writer(&mut buf).await.unwrap();
    let mut buf = Cursor::new(buf);
    let (size, decoded) = NodeId::decode_from_async_reader(&mut buf).await.unwrap();
    assert!(NodeId::decode_from_async_reader(&mut Cursor::new(&[]))
      .await
      .is_err());
    assert!(
      NodeId::decode_from_async_reader(&mut Cursor::new(&[255, 255]))
        .await
        .is_err()
    );
    assert_eq!(size, id.encoded_len());
    assert_eq!(id, decoded);

    let id = NodeId::random(32);
    let mut buf = vec![0; id.encoded_len()];
    id.encode(&mut buf).unwrap();
    let mut buf = Cursor::new(buf);
    let (size, decoded) = NodeId::decode_from_async_reader(&mut buf).await.unwrap();
    assert_eq!(size, id.encoded_len());
    assert_eq!(id, decoded);

    let id = NodeId::random(96);
    let mut buf = vec![0; id.encoded_len()];
    id.encode(&mut buf).unwrap();
    let mut buf = Cursor::new(buf);
    let (size, decoded) = NodeId::decode_from_async_reader(&mut buf).await.unwrap();
    assert_eq!(size, id.encoded_len());
    assert_eq!(id, decoded);

    let id = NodeId::random(32);
    let mut buf = Vec::new();
    id.encode_to_async_writer(&mut buf).await.unwrap();
    let (size, decoded) = NodeId::decode(&buf).unwrap();
    assert_eq!(size, id.encoded_len());
    assert_eq!(id, decoded);

    let id = NodeId::random(96);
    let mut buf = Vec::new();
    id.encode_to_writer(&mut buf).unwrap();
    let (size, decoded) = NodeId::decode(&buf).unwrap();
    assert_eq!(size, id.encoded_len());
    assert_eq!(id, decoded);
  }

  #[cfg(feature = "serde")]
  #[test]
  fn test_serde() {
    let id = NodeId::random(32);
    let s = serde_json::to_string(&id).unwrap();
    let decoded: NodeId = serde_json::from_str(&s).unwrap();
    assert_eq!(id, decoded);
  }
}
