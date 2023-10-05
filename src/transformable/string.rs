use super::*;

macro_rules! impl_string {
  ($ty: ty) => {
    #[cfg_attr(all(feature = "async", feature = "std"), async_trait::async_trait)]
    impl Transformable for $ty {
      type Error = StringTransformableError;

      fn encode(&self, dst: &mut [u8]) -> Result<(), Self::Error> {
        encode_bytes(self.as_str().as_bytes(), dst)
          .map_err(StringTransformableError::from_bytes_error)
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
        encode_bytes_to(self.as_str().as_bytes(), dst)
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
        encode_bytes_to_async(self.as_str().as_bytes(), dst).await
      }

      fn encoded_len(&self) -> usize {
        encoded_bytes_len(self.as_str().as_bytes())
      }

      fn decode(src: &[u8]) -> Result<Self, Self::Error>
      where
        Self: Sized,
      {
        decode_bytes(src)
          .map_err(StringTransformableError::from_bytes_error)
          .and_then(|bytes| {
            core::str::from_utf8(bytes.as_ref())
              .map(Self::from)
              .map_err(Into::into)
          })
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
        decode_bytes_from(src).and_then(|bytes| {
          core::str::from_utf8(bytes.as_ref())
            .map(Self::from)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
        })
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
        decode_bytes_from_async(src).await.and_then(|bytes| {
          core::str::from_utf8(bytes.as_ref())
            .map(Self::from)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
        })
      }
    }
  };
}

impl_string!(String);
impl_string!(smol_str::SmolStr);
