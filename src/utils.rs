#[cfg(feature = "std")]
#[inline]
pub(crate) fn invalid_data<E: core::error::Error + Send + Sync + 'static>(e: E) -> std::io::Error {
  std::io::Error::new(std::io::ErrorKind::InvalidData, e)
}
