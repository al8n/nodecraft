use super::*;
use core::borrow::Borrow;

macro_rules! impl_string {
  ($ty: ty => $test_fn:ident($init: expr)) => {
    impl Transformable for $ty {
      type Error = StringTransformError;

      fn encode(&self, dst: &mut [u8]) -> Result<(), Self::Error> {
        let src: &str = self.borrow();
        encode_bytes(src.as_bytes(), dst).map_err(StringTransformError::from_bytes_error)
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
        let src: &str = self.borrow();
        encode_bytes_to(src.as_bytes(), dst)
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
        let src: &str = self.borrow();
        encode_bytes_to_async(src.as_bytes(), dst).await
      }

      fn encoded_len(&self) -> usize {
        let src: &str = self.borrow();
        encoded_bytes_len(src.as_bytes())
      }

      fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error>
      where
        Self: Sized,
      {
        decode_bytes(src)
          .map_err(StringTransformError::from_bytes_error)
          .and_then(|(readed, bytes)| {
            core::str::from_utf8(bytes.as_ref())
              .map(|s| (readed, Self::from(s)))
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
      fn decode_from_reader<R: std::io::Read>(src: &mut R) -> std::io::Result<(usize, Self)>
      where
        Self: Sized,
      {
        decode_bytes_from(src).and_then(|(readed, bytes)| {
          core::str::from_utf8(bytes.as_ref())
            .map(|s| (readed, Self::from(s)))
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
      ) -> std::io::Result<(usize, Self)>
      where
        Self: Sized,
      {
        decode_bytes_from_async(src)
          .await
          .and_then(|(readed, bytes)| {
            core::str::from_utf8(bytes.as_ref())
              .map(|s| (readed, Self::from(s)))
              .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
          })
      }
    }
  
    test_transformable!($ty => $test_fn($init));
  };
}

impl_string!(String => test_string_transformable("hello world".to_string()));
impl_string!(smol_str::SmolStr => test_smol_str_transformable(smol_str::SmolStr::from("hello world")));
impl_string!(Box<str> => test_box_str_transformable(Box::from("hello world")));
impl_string!(Arc<str> => test_arc_str_transformable(Arc::from("hello world")));
