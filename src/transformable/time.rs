use core::{mem, time::Duration};

use super::Transformable;

const ENCODED_LEN: usize = mem::size_of::<u64>() + mem::size_of::<u32>();

/// Error returned by [`Instant`] or [`Duration`] when transforming.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeTransformError {
  /// The buffer is too small to encode the value.
  EncodeBufferTooSmall,
  /// Corrupted binary data.
  Corrupted,
  /// Similar to [`SystemTimeError`](std::time::SystemTimeError).
  Elapsed(Duration),
}

impl core::fmt::Display for TimeTransformError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::EncodeBufferTooSmall => write!(
        f,
        "buffer is too small, use `Transformable::encoded_len` to pre-allocate a buffer with enough space"
      ),
      Self::Corrupted => write!(f, "corrupted binary data"),
      Self::Elapsed(_) => write!(f, "elapsed time"),
    }
  }
}

#[cfg(feature = "std")]
impl std::error::Error for TimeTransformError {}

impl Transformable for Duration {
  type Error = TimeTransformError;

  fn encode(&self, dst: &mut [u8]) -> Result<(), Self::Error> {
    if dst.len() < self.encoded_len() {
      return Err(TimeTransformError::EncodeBufferTooSmall);
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
      return Err(TimeTransformError::Corrupted);
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
mod _impl {
  use std::time::{Instant, SystemTime};

  use super::*;

  impl Transformable for Instant {
    type Error = TimeTransformError;

    fn encode(&self, dst: &mut [u8]) -> Result<(), Self::Error> {
      if dst.len() < self.encoded_len() {
        return Err(TimeTransformError::EncodeBufferTooSmall);
      }

      let buf = encode_duration_unchecked(self.elapsed());
      dst[..ENCODED_LEN].copy_from_slice(&buf);
      Ok(())
    }

    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn encode_to_writer<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
      let buf = encode_duration_unchecked(self.elapsed());
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

      let buf = encode_duration_unchecked(self.elapsed());
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
        return Err(TimeTransformError::Corrupted);
      }

      let (readed, dur) = decode_duration_unchecked(src);
      Ok((readed, Instant::now() - dur))
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
      Ok((readed, Instant::now() - dur))
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
      Ok((readed, Instant::now() - dur))
    }
  }

  impl Transformable for SystemTime {
    type Error = TimeTransformError;

    fn encode(&self, dst: &mut [u8]) -> Result<(), Self::Error> {
      if dst.len() < self.encoded_len() {
        return Err(TimeTransformError::EncodeBufferTooSmall);
      }

      let buf = encode_duration_unchecked(self.elapsed().map_err(|e| TimeTransformError::Elapsed(e.duration()))?);
      dst[..ENCODED_LEN].copy_from_slice(&buf);
      Ok(())
    }

    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn encode_to_writer<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
      let buf = encode_duration_unchecked(self.elapsed().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?);
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

      let buf = encode_duration_unchecked(self.elapsed().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?);
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
        return Err(TimeTransformError::Corrupted);
      }

      let (readed, dur) = decode_duration_unchecked(src);
      Ok((readed, SystemTime::now() - dur))
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
      Ok((readed, SystemTime::now() - dur))
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
      Ok((readed, SystemTime::now() - dur))
    }
  }
}
