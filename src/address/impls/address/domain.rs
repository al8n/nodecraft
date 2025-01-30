use core::{fmt, hash::Hash};

use smol_str03::SmolStr;

/// A type which encapsulates a string (borrowed or owned) that is a syntactically valid DNS name.
#[derive(Clone, Debug, Eq)]
#[cfg_attr(
  feature = "rkyv",
  derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(
  feature = "rkyv",
  rkyv(compare(PartialEq), derive(PartialEq, Eq, PartialOrd, Ord, Hash))
)]
pub(crate) struct Domain(SmolStr);

#[cfg(feature = "serde")]
const _: () = {
  impl serde::Serialize for Domain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: serde::Serializer,
    {
      self.as_str().serialize(serializer)
    }
  }

  impl<'de> serde::Deserialize<'de> for Domain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: serde::Deserializer<'de>,
    {
      let s = <&str>::deserialize(deserializer)?;
      s.try_into().map_err(serde::de::Error::custom)
    }
  }
};

impl core::fmt::Display for Domain {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.as_str().fmt(f)
  }
}

impl core::borrow::Borrow<str> for Domain {
  fn borrow(&self) -> &str {
    self.as_str()
  }
}

impl PartialEq for Domain {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.as_str() == other.as_str()
  }
}

impl PartialOrd for Domain {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Domain {
  #[inline]
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    self.as_str().cmp(other.as_str())
  }
}

impl core::hash::Hash for Domain {
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    self.as_str().hash(state)
  }
}

impl Domain {
  /// Add const constructor for static DNS names
  pub const fn new_static(s: &'static str) -> Result<Self, InvalidDomainError> {
    // Validate at compile time
    match validate(s.as_bytes()) {
      Ok(()) => Ok(Self(SmolStr::new_static(s))),
      Err(e) => Err(e),
    }
  }

  /// Returns the str representation.
  #[inline]
  pub fn as_str(&self) -> &str {
    self.0.trim_end_matches('.')
  }

  /// Returns the fully-qualified domain name representation.
  #[inline]
  pub fn fqdn_str(&self) -> &str {
    self.0.as_str()
  }

  fn try_from_inner(value: &[u8], validated: bool) -> Result<Self, InvalidDomainError> {
    if !validated {
      validate(value)?;
    }

    // Early return if already has trailing dot
    if value.ends_with(b".") {
      return Ok(Self(
        // SAFETY: We know the input is valid UTF-8 from validation
        unsafe { core::str::from_utf8_unchecked(value) }.into(),
      ));
    }

    let len = value.len();
    if len + 1 < 23 {
      // Use stack allocation for small strings
      let mut buf = [0u8; 23];
      buf[..len].copy_from_slice(value);
      buf[len] = b'.'; // Add trailing dot
      Ok(Self(
        // SAFETY: We know the input is valid UTF-8 from validation
        unsafe { core::str::from_utf8_unchecked(&buf[..=len]) }.into(),
      ))
    } else {
      // Consider pre-allocating with capacity
      let mut string = String::with_capacity(value.len() + 1);
      // SAFETY: We know the input is valid UTF-8 from validation
      string.push_str(unsafe { core::str::from_utf8_unchecked(value) });
      string.push('.');
      Ok(Self(string.into()))
    }
  }
}

#[cfg(feature = "alloc")]
impl TryFrom<String> for Domain {
  type Error = InvalidDomainError;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    // String is guaranteed to be valid UTF-8, but we need to validate DNS rules
    Self::try_from_inner(value.as_bytes(), false)
  }
}

#[cfg(feature = "alloc")]
impl TryFrom<&String> for Domain {
  type Error = InvalidDomainError;

  fn try_from(value: &String) -> Result<Self, Self::Error> {
    value.as_str().try_into()
  }
}

impl<'a> TryFrom<&'a str> for Domain {
  type Error = InvalidDomainError;

  fn try_from(value: &'a str) -> Result<Self, Self::Error> {
    // str is guaranteed to be valid UTF-8, but we need to validate DNS rules
    Self::try_from_inner(value.as_bytes(), false)
  }
}

impl core::str::FromStr for Domain {
  type Err = InvalidDomainError;

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    value.try_into()
  }
}

impl<'a> TryFrom<&'a [u8]> for Domain {
  type Error = InvalidDomainError;

  fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
    // bytes is guaranteed to be valid UTF-8, but we need to validate DNS rules
    Self::try_from_inner(value, true)
  }
}

impl AsRef<str> for Domain {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

/// The provided input could not be parsed because
/// it is not a syntactically-valid DNS Domain.
#[derive(Debug)]
pub struct InvalidDomainError;

impl fmt::Display for InvalidDomainError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str("invalid dns name")
  }
}

impl core::error::Error for InvalidDomainError {}

const fn validate(input: &[u8]) -> Result<(), InvalidDomainError> {
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
  if input.len() > MAX_NAME_LENGTH {
    return Err(InvalidDomainError);
  }

  let mut i = 0;
  while i < len {
    let ch = input[i];
    state = match (state, ch) {
      (Start | Next | NextAfterNumericOnly | Hyphen { .. }, b'.') => {
        return Err(InvalidDomainError)
      }
      (Subsequent { .. }, b'.') => Next,
      (NumericOnly { .. }, b'.') => NextAfterNumericOnly,
      (Subsequent { len } | NumericOnly { len } | Hyphen { len }, _) if len >= MAX_LABEL_LENGTH => {
        return Err(InvalidDomainError)
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
      _ => return Err(InvalidDomainError),
    };
    i += 1;
  }

  if matches!(
    state,
    Start | Hyphen { .. } | NumericOnly { .. } | NextAfterNumericOnly
  ) {
    return Err(InvalidDomainError);
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[cfg(feature = "alloc")]
  static TESTS: &[(&str, bool)] = &[
    ("", false),
    ("localhost", true),
    ("LOCALHOST", true),
    (".localhost", false),
    ("..localhost", false),
    ("1.2.3.4", false),
    ("127.0.0.1", false),
    ("absolute.", true),
    ("absolute..", false),
    ("multiple.labels.absolute.", true),
    ("foo.bar.com", true),
    ("infix-hyphen-allowed.com", true),
    ("-prefixhypheninvalid.com", false),
    ("suffixhypheninvalid--", false),
    ("suffixhypheninvalid-.com", false),
    ("foo.lastlabelendswithhyphen-", false),
    ("infix_underscore_allowed.com", true),
    ("_prefixunderscorevalid.com", true),
    ("labelendswithnumber1.bar.com", true),
    ("xn--bcher-kva.example", true),
    (
        "sixtythreesixtythreesixtythreesixtythreesixtythreesixtythreesix.com",
        true,
    ),
    (
        "sixtyfoursixtyfoursixtyfoursixtyfoursixtyfoursixtyfoursixtyfours.com",
        false,
    ),
    (
        "012345678901234567890123456789012345678901234567890123456789012.com",
        true,
    ),
    (
        "0123456789012345678901234567890123456789012345678901234567890123.com",
        false,
    ),
    (
        "01234567890123456789012345678901234567890123456789012345678901-.com",
        false,
    ),
    (
        "012345678901234567890123456789012345678901234567890123456789012-.com",
        false,
    ),
    ("numeric-only-final-label.1", false),
    ("numeric-only-final-label.absolute.1.", false),
    ("1starts-with-number.com", true),
    ("1Starts-with-number.com", true),
    ("1.2.3.4.com", true),
    ("123.numeric-only-first-label", true),
    ("a123b.com", true),
    ("numeric-only-middle-label.4.com", true),
    ("1000-sans.badssl.com", true),
    ("twohundredandfiftythreecharacters.twohundredandfiftythreecharacters.twohundredandfiftythreecharacters.twohundredandfiftythreecharacters.twohundredandfiftythreecharacters.twohundredandfiftythreecharacters.twohundredandfiftythreecharacters.twohundredandfi", true),
    ("twohundredandfiftyfourcharacters.twohundredandfiftyfourcharacters.twohundredandfiftyfourcharacters.twohundredandfiftyfourcharacters.twohundredandfiftyfourcharacters.twohundredandfiftyfourcharacters.twohundredandfiftyfourcharacters.twohundredandfiftyfourc", false),
  ];

  #[cfg(feature = "alloc")]
  #[test]
  fn test_validation() {
    for (input, expected) in TESTS {
      #[cfg(feature = "std")]
      println!("test: {:?} expected valid? {:?}", input, expected);
      let name_ref = Domain::try_from(*input);
      assert_eq!(*expected, name_ref.is_ok());
      let name = Domain::try_from(input.to_string());
      assert_eq!(*expected, name.is_ok());
    }
  }

  #[cfg(feature = "alloc")]
  #[test]
  fn test_basic() {
    let name = Domain::try_from(&"localhost".to_string()).unwrap();
    assert_eq!("localhost", name.as_ref());
    let err = InvalidDomainError;
    println!("{}", err);
  }

  #[cfg(feature = "std")]
  #[test]
  fn test_borrow() {
    use std::collections::HashSet;
    let name = Domain::try_from("localhost").unwrap();
    let mut set = HashSet::new();
    set.insert(name);

    assert!(set.contains("localhost"));
  }

  #[test]
  fn test_try_from_bytes() {
    use super::Domain;

    let name = Domain::try_from(b"localhost".as_slice()).unwrap();
    assert_eq!("localhost", name.as_str());
    assert_eq!("localhost.", name.fqdn_str());

    let name = Domain::try_from(b"localhost.".as_slice()).unwrap();
    assert_eq!("localhost", name.as_str());

    let name = Domain::try_from(b"labelendswithnumber1.bar.com".as_slice()).unwrap();
    assert_eq!(name.to_string().as_str(), "labelendswithnumber1.bar.com");

    let name = Domain::try_from(b"labelendswithnumber1.bar.com.".as_slice()).unwrap();
    assert_eq!(name.to_string().as_str(), "labelendswithnumber1.bar.com");
  }
}
