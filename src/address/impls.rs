#[cfg(any(feature = "std", feature = "alloc"))]
mod address;
#[cfg(any(feature = "std", feature = "alloc"))]
mod address_ref;
#[cfg(any(feature = "std", feature = "alloc"))]
mod domain;
#[cfg(any(feature = "std", feature = "alloc"))]
mod domain_ref;

#[cfg(any(feature = "std", feature = "alloc"))]
pub use address::*;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use address_ref::*;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use domain::*;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use domain_ref::*;

/// An error which can be returned when parsing a [`HostAddr`].
#[derive(Debug, thiserror::Error)]
#[cfg(any(feature = "std", feature = "alloc"))]
pub enum ParseHostAddrError {
  /// Returned if the provided str is missing port.
  #[error("address is missing port")]
  PortNotFound,
  /// Returned if the provided str is not a valid address.
  #[error(transparent)]
  Domain(#[from] ParseDomainError),
  /// Returned if the provided str is not a valid port.
  #[error("invalid port: {0}")]
  Port(#[from] core::num::ParseIntError),
}

/// The provided input could not be parsed because
/// it is not a syntactically-valid DNS Domain.
#[derive(Debug, thiserror::Error)]
#[error("invalid domain name")]
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct ParseDomainError;

#[cfg(any(feature = "std", feature = "alloc"))]
impl ParseDomainError {
  /// Returns the error message.
  #[inline]
  pub const fn as_str(&self) -> &'static str {
    "invalid domain name"
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
const fn validate(input: &[u8]) -> Result<(), ParseDomainError> {
  enum State {
    Start,
    Next,
    NumericOnly { len: usize },
    NextAfterNumericOnly,
    Subsequent { len: usize },
    Hyphen { len: usize },
  }

  use State::*;

  let mut state = Start;

  /// "Labels must be 63 characters or less."
  const MAX_LABEL_LENGTH: usize = 63;

  /// https://devblogs.microsoft.com/oldnewthing/20120412-00/?p=7873
  const MAX_NAME_LENGTH: usize = 253;

  let len = input.len();
  if len > MAX_NAME_LENGTH || len == 0 {
    return Err(ParseDomainError);
  }

  if input[0] == b'.' && len == 1 {
    return Ok(());
  }

  let mut i = 0;
  while i < len {
    let ch = input[i];
    state = match (state, ch) {
      (Start | Next | NextAfterNumericOnly | Hyphen { .. }, b'.') => return Err(ParseDomainError),
      (Subsequent { .. }, b'.') => Next,
      (NumericOnly { .. }, b'.') => NextAfterNumericOnly,
      (Subsequent { len } | NumericOnly { len } | Hyphen { len }, _) if len >= MAX_LABEL_LENGTH => {
        return Err(ParseDomainError);
      }
      (Start | Next | NextAfterNumericOnly, b'0'..=b'9') => NumericOnly { len: 1 },
      (NumericOnly { len }, b'0'..=b'9') => NumericOnly { len: len + 1 },
      (Start | Next | NextAfterNumericOnly, b'a'..=b'z' | b'A'..=b'Z' | b'_') => {
        Subsequent { len: 1 }
      }
      (Subsequent { len } | NumericOnly { len } | Hyphen { len }, b'-') => Hyphen { len: len + 1 },
      (
        Subsequent { len } | NumericOnly { len } | Hyphen { len },
        b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9',
      ) => Subsequent { len: len + 1 },
      _ => return Err(ParseDomainError),
    };
    i += 1;
  }

  if matches!(
    state,
    Start | Hyphen { .. } | NumericOnly { .. } | NextAfterNumericOnly
  ) {
    return Err(ParseDomainError);
  }

  Ok(())
}

#[test]
fn test_error() {
  assert_eq!(ParseDomainError.as_str(), "invalid domain name");
}
