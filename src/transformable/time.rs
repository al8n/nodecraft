use core::{mem, time::Duration};

use super::Transformable;

const ENCODED_LEN: usize = mem::size_of::<u64>() + mem::size_of::<u32>();

/// Error returned by [`Duration`] when transforming.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DurationTransformError {
  /// The buffer is too small to encode the value.
  EncodeBufferTooSmall,
  /// Corrupted binary data.
  Corrupted,
}

impl core::fmt::Display for DurationTransformError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::EncodeBufferTooSmall => write!(
        f,
        "buffer is too small, use `Transformable::encoded_len` to pre-allocate a buffer with enough space"
      ),
      Self::Corrupted => write!(f, "corrupted binary data"),
    }
  }
}

#[cfg(feature = "std")]
impl std::error::Error for DurationTransformError {}

impl Transformable for Duration {
  type Error = DurationTransformError;

  fn encode(&self, dst: &mut [u8]) -> Result<(), Self::Error> {
    if dst.len() < self.encoded_len() {
      return Err(Self::Error::EncodeBufferTooSmall);
    }

    let buf = encode_duration_unchecked(*self);
    dst[..ENCODED_LEN].copy_from_slice(&buf);
    Ok(())
  }

  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn encode_to_writer<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
    let buf = encode_duration_unchecked(*self);
    writer.write_all(&buf)
  }

  #[cfg(all(feature = "async", feature = "std"))]
  #[cfg_attr(docsrs, doc(cfg(all(feature = "async", feature = "std"))))]
  async fn encode_to_async_writer<W: futures::io::AsyncWrite + Send + Unpin>(
    &self,
    writer: &mut W,
  ) -> std::io::Result<()>
  where
    Self::Error: Send + Sync + 'static,
  {
    use futures::AsyncWriteExt;

    let buf = encode_duration_unchecked(*self);
    writer.write_all(&buf).await
  }

  fn encoded_len(&self) -> usize {
    ENCODED_LEN
  }

  fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error>
  where
    Self: Sized,
  {
    if src.len() < ENCODED_LEN {
      return Err(Self::Error::Corrupted);
    }

    Ok(decode_duration_unchecked(src))
  }

  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn decode_from_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<(usize, Self)>
  where
    Self: Sized,
  {
    let mut buf = [0; ENCODED_LEN];
    reader.read_exact(&mut buf)?;
    Ok(decode_duration_unchecked(&buf))
  }

  #[cfg(all(feature = "async", feature = "std"))]
  #[cfg_attr(docsrs, doc(cfg(all(feature = "async", feature = "std"))))]
  async fn decode_from_async_reader<R: futures::io::AsyncRead + Send + Unpin>(
    reader: &mut R,
  ) -> std::io::Result<(usize, Self)>
  where
    Self: Sized,
    Self::Error: Send + Sync + 'static,
  {
    use futures::AsyncReadExt;

    let mut buf = [0; ENCODED_LEN];
    reader.read_exact(&mut buf).await?;
    Ok(decode_duration_unchecked(&buf))
  }
}

#[inline]
const fn encode_duration_unchecked(dur: Duration) -> [u8; ENCODED_LEN] {
  let secs = dur.as_secs().to_be_bytes();
  let nanos = dur.subsec_nanos().to_be_bytes();
  [
    secs[0], secs[1], secs[2], secs[3], secs[4], secs[5], secs[6], secs[7], nanos[0], nanos[1],
    nanos[2], nanos[3],
  ]
}

#[inline]
const fn decode_duration_unchecked(src: &[u8]) -> (usize, Duration) {
  let secs = u64::from_be_bytes([
    src[0], src[1], src[2], src[3], src[4], src[5], src[6], src[7],
  ]);
  let nanos = u32::from_be_bytes([src[8], src[9], src[10], src[11]]);
  (ENCODED_LEN, Duration::new(secs, nanos))
}

#[cfg(feature = "std")]
pub use _impl::*;

#[cfg(feature = "std")]
mod _impl {
  use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};

  use super::*;

  /// Error returned by [`SystemTime`] when transforming.
  #[derive(Debug, Clone)]
  pub enum SystemTimeTransformError {
    /// The buffer is too small to encode the value.
    EncodeBufferTooSmall,
    /// Corrupted binary data.
    Corrupted,
    /// Invalid system time.
    InvalidSystemTime(SystemTimeError),
  }

  impl core::fmt::Display for SystemTimeTransformError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
      Self::EncodeBufferTooSmall => write!(
        f,
        "buffer is too small, use `Transformable::encoded_len` to pre-allocate a buffer with enough space"
      ),
      Self::Corrupted => write!(f, "corrupted binary data"),
      Self::InvalidSystemTime(e) => write!(f, "{e}"),
    }
    }
  }

  #[cfg(feature = "std")]
  impl std::error::Error for SystemTimeTransformError {}

  impl Transformable for SystemTime {
    type Error = SystemTimeTransformError;

    fn encode(&self, dst: &mut [u8]) -> Result<(), Self::Error> {
      if dst.len() < self.encoded_len() {
        return Err(Self::Error::EncodeBufferTooSmall);
      }

      let buf = encode_duration_unchecked(
        self
          .duration_since(UNIX_EPOCH)
          .map_err(Self::Error::InvalidSystemTime)?,
      );
      dst[..ENCODED_LEN].copy_from_slice(&buf);
      Ok(())
    }

    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn encode_to_writer<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
      let mut buf = [0u8; ENCODED_LEN];
      self
        .encode(&mut buf)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
      writer.write_all(&buf)
    }

    #[cfg(all(feature = "async", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "async", feature = "std"))))]
    async fn encode_to_async_writer<W: futures::io::AsyncWrite + Send + Unpin>(
      &self,
      writer: &mut W,
    ) -> std::io::Result<()>
    where
      Self::Error: Send + Sync + 'static,
    {
      use futures::AsyncWriteExt;

      let mut buf = [0u8; ENCODED_LEN];
      self
        .encode(&mut buf)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
      writer.write_all(&buf).await
    }

    fn encoded_len(&self) -> usize {
      ENCODED_LEN
    }

    fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error>
    where
      Self: Sized,
    {
      if src.len() < ENCODED_LEN {
        return Err(Self::Error::Corrupted);
      }

      let (readed, dur) = decode_duration_unchecked(src);
      Ok((readed, UNIX_EPOCH + dur))
    }

    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn decode_from_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<(usize, Self)>
    where
      Self: Sized,
    {
      let mut buf = [0; ENCODED_LEN];
      reader.read_exact(&mut buf)?;
      let (readed, dur) = decode_duration_unchecked(&buf);
      Ok((readed, UNIX_EPOCH + dur))
    }

    #[cfg(all(feature = "async", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "async", feature = "std"))))]
    async fn decode_from_async_reader<R: futures::io::AsyncRead + Send + Unpin>(
      reader: &mut R,
    ) -> std::io::Result<(usize, Self)>
    where
      Self: Sized,
      Self::Error: Send + Sync + 'static,
    {
      use futures::AsyncReadExt;

      let mut buf = [0; ENCODED_LEN];
      reader.read_exact(&mut buf).await?;
      let (readed, dur) = decode_duration_unchecked(&buf);
      Ok((readed, UNIX_EPOCH + dur))
    }
  }

  #[tokio::test]
  async fn test_systemtime_transformable() {
    let now = SystemTime::now();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let mut buf = [0; ENCODED_LEN];
    now.encode(&mut buf).unwrap();
    let (_, decoded) = SystemTime::decode(&buf).unwrap();
    assert_eq!(decoded, now);

    let mut buf = Vec::new();
    now.encode_to_writer(&mut buf).unwrap();
    let (_, decoded) = SystemTime::decode_from_reader(&mut buf.as_slice()).unwrap();
    assert_eq!(decoded, now);

    let mut buf = Vec::new();
    now.encode_to_async_writer(&mut buf).await.unwrap();
    let (_, decoded) = SystemTime::decode_from_async_reader(&mut buf.as_slice())
      .await
      .unwrap();
    assert_eq!(decoded, now);
  }
}

#[tokio::test]
async fn test_duration_transformable() {
  let now = Duration::new(10, 1080);
  std::thread::sleep(std::time::Duration::from_millis(10));
  let mut buf = [0; ENCODED_LEN];
  now.encode(&mut buf).unwrap();
  let (_, decoded) = Duration::decode(&buf).unwrap();
  assert_eq!(decoded, now);

  let mut buf = Vec::new();
  now.encode_to_writer(&mut buf).unwrap();
  let (_, decoded) = Duration::decode_from_reader(&mut buf.as_slice()).unwrap();
  assert_eq!(decoded, now);

  let mut buf = Vec::new();
  now.encode_to_async_writer(&mut buf).await.unwrap();
  let (_, decoded) = Duration::decode_from_async_reader(&mut buf.as_slice())
    .await
    .unwrap();
  assert_eq!(decoded, now);
}
