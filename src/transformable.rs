#[cfg(feature = "bytes")]
mod bytes;
#[cfg(feature = "alloc")]
mod string;
#[cfg(feature = "alloc")]
mod vec;

#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

#[cfg(not(feature = "std"))]
use ::alloc::{string::String, vec::Vec};

/// The type can transform its representation between structured and byte form.
#[cfg_attr(all(feature = "async", feature = "std"), async_trait::async_trait)]
pub trait Transformable {
  /// The error type returned when encoding or decoding fails.
  #[cfg(feature = "std")]
  type Error: std::error::Error + Send + Sync + 'static;

  /// The error type returned when encoding or decoding fails.
  #[cfg(not(feature = "std"))]
  type Error: core::fmt::Display + Send + Sync + 'static;

  /// Encodes the value into the given buffer for transmission.
  fn encode(&self, dst: &mut [u8]) -> Result<(), Self::Error>;

  /// Encodes the value into the given writer for transmission.
  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn encode_to_writer<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()>;

  /// Encodes the value into the given async writer for transmission.
  #[cfg(all(feature = "async", feature = "std"))]
  #[cfg_attr(docsrs, doc(cfg(all(feature = "async", feature = "std"))))]
  async fn encode_to_async_writer<W: futures::io::AsyncWrite + Send + Unpin>(
    &self,
    writer: &mut W,
  ) -> std::io::Result<()>;

  /// Returns the encoded length of the value.
  /// This is used to pre-allocate a buffer for encoding.
  fn encoded_len(&self) -> usize;

  /// Decodes the value from the given buffer received over the wire.
  fn decode(src: &[u8]) -> Result<Self, Self::Error>
  where
    Self: Sized;

  /// Decodes the value from the given reader received over the wire.
  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn decode_from_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self>
  where
    Self: Sized;

  /// Decodes the value from the given async reader received over the wire.
  #[cfg(all(feature = "async", feature = "std"))]
  #[cfg_attr(docsrs, doc(cfg(all(feature = "async", feature = "std"))))]
  async fn decode_from_async_reader<R: futures::io::AsyncRead + Send + Unpin>(
    reader: &mut R,
  ) -> std::io::Result<Self>
  where
    Self: Sized;
}

/// The error type for errors that get returned when encoding or decoding fails.
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum BytesTransformableError {
  /// Returned when the buffer is too small to encode.
  #[cfg_attr(feature = "std", error(
    "buffer is too small, use `Transformable::encoded_len` to pre-allocate a buffer with enough space"
  ))]
  EncodeBufferTooSmall,
  /// Returned when the bytes are corrupted.
  #[cfg_attr(feature = "std", error("corrupted"))]
  Corrupted,
  /// Returned when there are some other errors.
  #[cfg(feature = "std")]
  #[cfg_attr(feature = "std", error("{0}"))]
  Custom(Box<dyn std::error::Error + Send + Sync + 'static>),
}

#[cfg(not(feature = "std"))]
impl core::fmt::Display for BytesTransformableError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::EncodeBufferTooSmall => write!(
        f,
        "buffer is too small, use `Transformable::encoded_len` to pre-allocate a buffer with enough space"
      ),
      Self::Corrupted => write!(f, "corrupted"),
    }
  }
}

impl BytesTransformableError {
  /// Create a new `BytesTransformableError::Corrupted` error.
  #[inline]
  pub const fn corrupted() -> Self {
    Self::Corrupted
  }

  /// Create a new `BytesTransformableError::Custom` error.
  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  #[inline]
  pub fn custom<E>(err: E) -> Self
  where
    E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
  {
    Self::Custom(err.into())
  }
}

/// The error type for errors that get returned when encoding or decoding str based structs fails.
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum StringTransformableError {
  /// Returned when the buffer is too small to encode.
  #[cfg_attr(feature = "std", error(
    "buffer is too small, use `Transformable::encoded_len` to pre-allocate a buffer with enough space"
  ))]
  EncodeBufferTooSmall,
  /// Returned when the decoding meet corruption.
  #[cfg_attr(feature = "std", error("corrupted"))]
  Corrupted,
  /// Returned when the decoding meet utf8 error.
  #[cfg_attr(feature = "std", error("{0}"))]
  Utf8Error(#[cfg_attr(feature = "std", from)] core::str::Utf8Error),
  /// Returned when there are some other errors.
  #[cfg(feature = "std")]
  #[cfg_attr(feature = "std", error("{0}"))]
  Custom(Box<dyn std::error::Error + Send + Sync + 'static>),
}

#[cfg(not(feature = "std"))]
impl core::convert::From<core::str::Utf8Error> for StringTransformableError {
  fn from(err: core::str::Utf8Error) -> Self {
    Self::Utf8Error(err)
  }
}

#[cfg(not(feature = "std"))]
impl core::fmt::Display for StringTransformableError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::EncodeBufferTooSmall => write!(
        f,
        "buffer is too small, use `Transformable::encoded_len` to pre-allocate a buffer with enough space"
      ),
      Self::Corrupted => write!(f, "corrupted"),
      Self::Utf8Error(val) => write!(f, "{val}"),
    }
  }
}

impl StringTransformableError {
  /// Create a new `BytesTransformableError::Corrupted` error.
  #[inline]
  pub const fn corrupted() -> Self {
    Self::Corrupted
  }

  /// Create a new `BytesTransformableError::Custom` error.
  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  #[inline]
  pub fn custom<E>(err: E) -> Self
  where
    E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
  {
    Self::Custom(err.into())
  }

  fn from_bytes_error(err: BytesTransformableError) -> Self {
    match err {
      BytesTransformableError::EncodeBufferTooSmall => Self::EncodeBufferTooSmall,
      BytesTransformableError::Corrupted => Self::Corrupted,
      #[cfg(feature = "std")]
      BytesTransformableError::Custom(err) => Self::Custom(err),
    }
  }
}

const LEGNTH_SIZE: usize = core::mem::size_of::<u32>();

#[cfg(all(feature = "std", feature = "async"))]
async fn decode_bytes_from_async<R: futures::io::AsyncRead + Unpin>(
  src: &mut R,
) -> std::io::Result<Vec<u8>> {
  use futures::AsyncReadExt;

  let mut len_buf = [0u8; LEGNTH_SIZE];
  src.read_exact(&mut len_buf).await?;
  let len = u32::from_be_bytes(len_buf) as usize;
  let mut buf = vec![0u8; len];
  src.read_exact(&mut buf).await.map(|_| buf)
}

#[cfg(feature = "std")]
fn decode_bytes_from<R: std::io::Read>(src: &mut R) -> std::io::Result<Vec<u8>> {
  let mut len_buf = [0u8; LEGNTH_SIZE];
  src.read_exact(&mut len_buf)?;
  let len = u32::from_be_bytes(len_buf) as usize;
  let mut buf = vec![0u8; len];
  src.read_exact(&mut buf).map(|_| buf)
}

fn decode_bytes(src: &[u8]) -> Result<Vec<u8>, BytesTransformableError> {
  let len = src.len();
  if len < core::mem::size_of::<u32>() {
    return Err(BytesTransformableError::Corrupted);
  }

  let len = u32::from_be_bytes([src[0], src[1], src[2], src[3]]) as usize;
  if len > len - core::mem::size_of::<u32>() {
    return Err(BytesTransformableError::Corrupted);
  }

  Ok(src[LEGNTH_SIZE..LEGNTH_SIZE + len].to_vec())
}

fn encode_bytes(src: &[u8], dst: &mut [u8]) -> Result<(), BytesTransformableError> {
  let encoded_len = src.len() + LEGNTH_SIZE;
  if dst.len() < encoded_len {
    return Err(BytesTransformableError::EncodeBufferTooSmall);
  }
  Ok(())
}

#[cfg(feature = "std")]
fn encode_bytes_to<W: std::io::Write>(src: &[u8], dst: &mut W) -> std::io::Result<()> {
  let len = src.len() as u32;
  dst
    .write_all(&len.to_be_bytes())
    .and_then(|_| dst.write_all(src))
}

#[cfg(all(feature = "std", feature = "async"))]
async fn encode_bytes_to_async<W: futures::io::AsyncWrite + Unpin>(
  src: &[u8],
  dst: &mut W,
) -> std::io::Result<()> {
  use futures::io::AsyncWriteExt;

  let len = src.len() as u32;
  dst.write_all(&len.to_be_bytes()).await?;
  dst.write_all(src).await
}

fn encoded_bytes_len(src: &[u8]) -> usize {
  core::mem::size_of::<u32>() + src.len()
}
