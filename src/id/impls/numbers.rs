use super::super::{NodeId, Transformable};

/// Error type for number based node id.
#[derive(Debug)]
pub enum NumberIdTransformableError {
  /// Returned when the buffer is too small to encode.
  EncodeBufferTooSmall,
  /// Returned when the id is corrupted.
  Corrupted,
}

impl core::fmt::Display for NumberIdTransformableError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::EncodeBufferTooSmall => write!(f, "buffer is too small, use `Transformable::encoded_len` to pre-allocate a buffer with enough space"),
      Self::Corrupted => write!(f, "corrupted id"),
    }
  }
}

#[cfg(feature = "std")]
impl std::error::Error for NumberIdTransformableError {}

macro_rules! impl_number_based_id {
  ($($ty: ty), + $(,)?) => {
    $(
      impl NodeId for $ty {}

      #[cfg_attr(all(feature = "async", feature = "std"), async_trait::async_trait)]
      impl Transformable for $ty {
        type Error = NumberIdTransformableError;

        fn encode(&self, dst: &mut [u8]) -> Result<(), Self::Error> {
          const SIZE: usize = core::mem::size_of::<$ty>();

          let encoded_len = self.encoded_len();
          if dst.len() < encoded_len {
            return Err(Self::Error::EncodeBufferTooSmall);
          }

          dst[..SIZE].copy_from_slice(&self.to_be_bytes());

          Ok(())
        }

        #[cfg(feature = "std")]
        #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
        fn encode_to_writer<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
          writer.write_all(&self.to_be_bytes())
        }

        #[cfg(all(feature = "async", feature = "std"))]
        #[cfg_attr(docsrs, doc(cfg(all(feature = "async", feature = "std"))))]
        async fn encode_to_async_writer<W: futures::io::AsyncWrite + Send + Unpin>(
          &self,
          writer: &mut W,
        ) -> std::io::Result<()> {
          use futures::AsyncWriteExt;

          writer.write_all(&self.to_be_bytes()).await
        }

        fn encoded_len(&self) -> usize {
          core::mem::size_of::<$ty>()
        }

        fn decode(src: &[u8]) -> Result<Self, Self::Error> where Self: Sized {
          const SIZE: usize = core::mem::size_of::<$ty>();

          if src.len() < SIZE {
            return Err(Self::Error::Corrupted);
          }

          let id = <$ty>::from_be_bytes((&src[..SIZE]).try_into().unwrap());
          Ok(id)
        }

        #[cfg(feature = "std")]
        #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
        fn decode_from_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> where Self: Sized {
          const SIZE: usize = core::mem::size_of::<$ty>();

          let mut buf = [0u8; SIZE];
          reader.read_exact(&mut buf)?;
          let id = <$ty>::from_be_bytes(buf);
          Ok(id)
        }

        #[cfg(all(feature = "async", feature = "std"))]
        #[cfg_attr(docsrs, doc(cfg(all(feature = "async", feature = "std"))))]
        async fn decode_from_async_reader<R: futures::io::AsyncRead + Send + Unpin>(
          reader: &mut R,
        ) -> std::io::Result<Self>
        where
          Self: Sized,
        {
          use futures::AsyncReadExt;

          const SIZE: usize = core::mem::size_of::<$ty>();

          let mut buf = [0u8; SIZE];
          reader.read_exact(&mut buf).await?;
          let id = <$ty>::from_be_bytes(buf);
          Ok(id)
        }
      }
    )+
  };
}

impl_number_based_id!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128,);
