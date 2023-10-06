use core::{borrow::Borrow, mem};

use smol_str::SmolStr;

use crate::{NodeId, Transformable};

#[cfg(feature = "std")]
use crate::utils::invalid_data;

/// Errors that can occur when transforming an `Id`.
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum IdTransformableError {
  /// Returned when the id is empty.
  #[cfg_attr(feature = "std", error("id cannot be empty"))]
  Empty,
  /// Returned when the id is too large.
  #[cfg_attr(
    feature = "std",
    error("id is too large, maximum size is 512 bytes, but got {0} bytes")
  )]
  TooLarge(usize),
  /// Returned when the buffer is too small to encode the [`Id`].
  #[cfg_attr(
    feature = "std",
    error("buffer is too small, use Id::encoded_size to pre-allocate a buffer with enough space")
  )]
  EncodeBufferTooSmall,
  /// Returned when the id is corrupted.
  #[cfg_attr(feature = "std", error("corrupted id"))]
  Corrupted,
  /// Returned when the id is not a valid utf8 string.
  #[cfg_attr(feature = "std", error("{0}"))]
  Utf8Error(#[cfg_attr(feature = "std", from)] core::str::Utf8Error),
}

#[cfg(not(feature = "std"))]
impl core::convert::From<core::str::Utf8Error> for IdTransformableError {
  fn from(err: core::str::Utf8Error) -> Self {
    Self::Utf8Error(err)
  }
}

#[cfg(not(feature = "std"))]
impl core::fmt::Display for IdTransformableError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Empty => write!(f, "id cannot be empty"),
      Self::TooLarge(num) => write!(
        f,
        "id is too large, maximum size is 512 bytes, but got {num} bytes"
      ),
      Self::EncodeBufferTooSmall => write!(
        f,
        "buffer is too small, use Id::encoded_size to pre-allocate a buffer with enough space"
      ),
      Self::Corrupted => write!(f, "corrupted id"),
      Self::Utf8Error(val) => write!(f, "{val}"),
    }
  }
}

/// A unique string identifying a server for all time.
/// The maximum length of an id is 512 bytes.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Id(SmolStr);

impl NodeId for Id {}

impl Id {
  /// The maximum length of an id is 512 bytes.
  pub const MAX_SIZE: usize = 512;

  /// Creates a new `Id` from the source.
  pub fn new<T: AsRef<str>>(src: T) -> Result<Self, IdTransformableError> {
    let src = src.as_ref();
    if src.is_empty() {
      return Err(IdTransformableError::Empty);
    }

    if src.len() > Self::MAX_SIZE {
      return Err(IdTransformableError::TooLarge(src.len()));
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
const INLINE: usize = 32;
const LENGTH_SIZE: usize = mem::size_of::<u16>();

#[cfg_attr(all(feature = "async", feature = "std"), async_trait::async_trait)]
impl Transformable for Id {
  type Error = IdTransformableError;

  fn encode(&self, dst: &mut [u8]) -> Result<(), Self::Error> {
    let encoded_len = self.encoded_len();
    if dst.len() < encoded_len {
      return Err(IdTransformableError::EncodeBufferTooSmall);
    }

    let mut cur = 0;
    dst[cur..cur + LENGTH_SIZE].copy_from_slice(&(self.0.len() as u16).to_be_bytes());
    cur += LENGTH_SIZE;
    dst[cur..cur + self.0.len()].copy_from_slice(self.0.as_bytes());
    Ok(())
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
  fn encode_to_writer<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
    let len = self.0.len() as u16;
    writer
      .write_all(&len.to_be_bytes())
      .and_then(|_| writer.write_all(self.0.as_str().as_bytes()))
  }

  /// Encodes the value into the given async writer.
  ///
  /// # Note
  /// The implementation of this method is not optimized, which means
  /// if your writer is expensive (e.g. `TcpStream`, `File`),
  /// it is better to use a [`BufWriter`](futures::io::BufWriter)
  /// to wrap your orginal writer to cut down the number of I/O times.
  #[cfg(all(feature = "async", feature = "std"))]
  #[cfg_attr(docsrs, doc(cfg(all(feature = "async", feature = "std"))))]
  async fn encode_to_async_writer<W: futures::io::AsyncWrite + Send + Unpin>(
    &self,
    writer: &mut W,
  ) -> std::io::Result<()> {
    use futures::AsyncWriteExt;

    let len = self.0.len() as u16;
    writer.write_all(&len.to_be_bytes()).await?;
    writer.write_all(self.0.as_str().as_bytes()).await
  }

  fn encoded_len(&self) -> usize {
    LENGTH_SIZE + self.0.len()
  }

  fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error>
  where
    Self: Sized,
  {
    if src.len() < LENGTH_SIZE {
      return Err(IdTransformableError::Corrupted);
    }

    let len = u16::from_be_bytes([src[0], src[1]]) as usize;
    if src.len() < LENGTH_SIZE + len {
      return Err(IdTransformableError::Corrupted);
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
    let len = u16::from_be_bytes(len_buf) as usize;
    if len == 0 {
      return Err(invalid_data(IdTransformableError::Empty));
    }

    if len > Self::MAX_SIZE {
      return Err(invalid_data(IdTransformableError::TooLarge(len)));
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
  #[cfg(all(feature = "async", feature = "std"))]
  #[cfg_attr(docsrs, doc(cfg(all(feature = "async", feature = "std"))))]
  async fn decode_from_async_reader<R: futures::io::AsyncRead + Send + Unpin>(
    reader: &mut R,
  ) -> std::io::Result<(usize, Self)>
  where
    Self: Sized,
  {
    use futures::AsyncReadExt;

    let mut len_buf = [0; core::mem::size_of::<u16>()];
    reader.read_exact(&mut len_buf).await?;
    let len = u16::from_be_bytes(len_buf) as usize;
    if len == 0 {
      return Err(invalid_data(IdTransformableError::Empty));
    }

    if len > Self::MAX_SIZE {
      return Err(invalid_data(IdTransformableError::TooLarge(len)));
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

impl core::str::FromStr for Id {
  type Err = IdTransformableError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::new(s)
  }
}

impl Borrow<str> for Id {
  fn borrow(&self) -> &str {
    self.as_str()
  }
}

impl AsRef<str> for Id {
  fn as_ref(&self) -> &str {
    self.0.as_ref()
  }
}

impl core::fmt::Display for Id {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.0.fmt(f)
  }
}

impl core::fmt::Debug for Id {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.0.fmt(f)
  }
}
