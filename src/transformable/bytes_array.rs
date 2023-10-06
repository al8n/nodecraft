use super::*;

#[cfg_attr(all(feature = "async", feature = "std"), async_trait::async_trait)]
impl<const N: usize> Transformable for [u8; N] {
  type Error = BytesTransformableError;

  fn encode(&self, dst: &mut [u8]) -> Result<(), Self::Error> {
    encode_bytes(self.as_ref(), dst)
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
    encode_bytes_to(self.as_ref(), dst)
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
    encode_bytes_to_async(self.as_ref(), dst).await
  }

  fn encoded_len(&self) -> usize {
    encoded_bytes_len(self.as_ref())
  }

  fn decode(src: &[u8]) -> Result<Self, Self::Error>
  where
    Self: Sized,
  {
    let len = src.len();
    if len != core::mem::size_of::<u32>() + N {
      return Err(BytesTransformableError::Corrupted);
    }

    let len = u32::from_be_bytes([src[0], src[1], src[2], src[3]]) as usize;
    if len != N {
      return Err(BytesTransformableError::Corrupted);
    }
    let mut buf = [0; N];
    buf.copy_from_slice(&src[LEGNTH_SIZE..LEGNTH_SIZE + N]);

    Ok(buf)
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
  fn decode_from_reader<R: std::io::Read>(src: &mut R) -> std::io::Result<Self>
  where
    Self: Sized,
  {
    use crate::utils::invalid_data;

    let mut len_buf = [0u8; LEGNTH_SIZE];
    src.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len != N {
      return Err(invalid_data(BytesTransformableError::Corrupted));
    }
    let mut buf = [0u8; N];
    src.read_exact(&mut buf).map(|_| buf)
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
  ) -> std::io::Result<Self>
  where
    Self: Sized,
  {
    use crate::utils::invalid_data;
    use futures::AsyncReadExt;

    let mut len_buf = [0u8; LEGNTH_SIZE];
    src.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len != N {
      return Err(invalid_data(BytesTransformableError::Corrupted));
    }
    let mut buf = [0u8; N];
    src.read_exact(&mut buf).await.map(|_| buf)
  }
}
