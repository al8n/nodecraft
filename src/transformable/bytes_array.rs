use super::*;

impl<const N: usize> Transformable for [u8; N] {
  type Error = BytesTransformableError;

  fn encode(&self, dst: &mut [u8]) -> Result<(), Self::Error> {
    if dst.len() < N {
      return Err(BytesTransformableError::EncodeBufferTooSmall);
    }

    dst[..N].copy_from_slice(self);
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
  fn encode_to_writer<W: std::io::Write>(&self, dst: &mut W) -> std::io::Result<()> {
    dst.write_all(self)
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
    dst: &mut W,
  ) -> std::io::Result<()> {
    use futures::AsyncWriteExt;

    dst.write_all(self).await
  }

  fn encoded_len(&self) -> usize {
    N
  }

  fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error>
  where
    Self: Sized,
  {
    let len = src.len();
    if len < N {
      return Err(BytesTransformableError::Corrupted);
    }

    let mut buf = [0; N];
    buf.copy_from_slice(&src[..N]);

    Ok((N, buf))
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
  fn decode_from_reader<R: std::io::Read>(src: &mut R) -> std::io::Result<(usize, Self)>
  where
    Self: Sized,
  {
    let mut buf = [0u8; N];
    src.read_exact(&mut buf).map(|_| (N, buf))
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
    src: &mut R,
  ) -> std::io::Result<(usize, Self)>
  where
    Self: Sized,
  {
    use futures::AsyncReadExt;

    let mut buf = [0u8; N];
    src.read_exact(&mut buf).await.map(|_| (N, buf))
  }
}
