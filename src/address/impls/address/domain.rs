use core::fmt;
use std::borrow::Cow;

use idna::{domain_to_ascii_cow, AsciiDenyList};
use smol_str03::SmolStr;

/// A type which encapsulates a string that is a syntactically domain name.
#[derive(Clone, Debug, Eq)]
#[cfg_attr(
  feature = "rkyv",
  derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(
  feature = "rkyv",
  rkyv(compare(PartialEq), derive(PartialEq, Eq, PartialOrd, Ord, Hash))
)]
pub struct Domain(SmolStr);

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

  /// Create a new Domain from a string, performing IDNA processing and validation.
  pub fn try_from_inner(domain: &[u8]) -> Result<Self, ParseDomainError> {
    let domain = if domain.is_ascii() {
      validate(domain)?;

      let domain = core::str::from_utf8(domain).expect("bytes must be valid utf8");
      // Early return if already has trailing dot
      if domain.ends_with('.') {
        return Ok(Self(domain.into()));
      }

      Cow::Borrowed(domain)
    } else {
      let without_dot = if domain.ends_with(b".") {
        &domain[..domain.len() - 1]
      } else {
        domain
      };
      let valid_domain =
        domain_to_ascii_cow(without_dot, AsciiDenyList::EMPTY).map_err(|_| ParseDomainError)?;

      if domain.ends_with(b".") && matches!(valid_domain, Cow::Borrowed(_)) {
        return Ok(Self(
          core::str::from_utf8(domain)
            .expect("bytes must be valid utf8")
            .into(),
        ));
      }

      valid_domain
    };

    let len = domain.len();
    if len + 1 < 23 {
      // Use stack allocation for small strings
      let mut buf = [0u8; 23];
      buf[..len].copy_from_slice(domain.as_bytes());
      buf[len] = b'.'; // Add trailing dot
      Ok(Self(
        // SAFETY: We know the input is valid UTF-8 from validation
        unsafe { core::str::from_utf8_unchecked(&buf[..=len]) }.into(),
      ))
    } else {
      // Consider pre-allocating with capacity
      let mut string = String::with_capacity(domain.len() + 1);
      string.push_str(&domain);
      string.push('.');
      Ok(Self(string.into()))
    }
  }
}

#[cfg(feature = "alloc")]
impl TryFrom<String> for Domain {
  type Error = ParseDomainError;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::try_from_inner(value.as_bytes())
  }
}

#[cfg(feature = "alloc")]
impl TryFrom<&String> for Domain {
  type Error = ParseDomainError;

  fn try_from(value: &String) -> Result<Self, Self::Error> {
    value.as_str().try_into()
  }
}

impl<'a> TryFrom<&'a str> for Domain {
  type Error = ParseDomainError;

  fn try_from(value: &'a str) -> Result<Self, Self::Error> {
    Self::try_from_inner(value.as_bytes())
  }
}

impl core::str::FromStr for Domain {
  type Err = ParseDomainError;

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    value.try_into()
  }
}

impl<'a> TryFrom<&'a [u8]> for Domain {
  type Error = ParseDomainError;

  fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
    Self::try_from_inner(value)
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
pub struct ParseDomainError;

impl fmt::Display for ParseDomainError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str("invalid domain name")
  }
}

impl core::error::Error for ParseDomainError {}

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
  if input.len() > MAX_NAME_LENGTH {
    return Err(ParseDomainError);
  }

  let mut i = 0;
  while i < len {
    let ch = input[i];
    state = match (state, ch) {
      (Start | Next | NextAfterNumericOnly | Hyphen { .. }, b'.') => return Err(ParseDomainError),
      (Subsequent { .. }, b'.') => Next,
      (NumericOnly { .. }, b'.') => NextAfterNumericOnly,
      (Subsequent { len } | NumericOnly { len } | Hyphen { len }, _) if len >= MAX_LABEL_LENGTH => {
        return Err(ParseDomainError)
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

#[cfg(test)]
mod tests {
  use core::str::FromStr;

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
    ("测试.com", true),
    ("测试.中国", true),
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
    let err = ParseDomainError;
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

  #[test]
  fn test_try_from_str() {
    use super::Domain;

    let name = Domain::try_from("localhost").unwrap();
    assert_eq!("localhost", name.as_str());
    assert_eq!("localhost.", name.fqdn_str());

    let name = Domain::from_str("localhost.").unwrap();
    assert_eq!("localhost", name.as_str());

    let name = Domain::from_str("labelendswithnumber1.bar.com").unwrap();
    assert_eq!(name.to_string().as_str(), "labelendswithnumber1.bar.com");

    let name = Domain::try_from("labelendswithnumber1.bar.com.").unwrap();
    assert_eq!(name.to_string().as_str(), "labelendswithnumber1.bar.com");
  }

  #[test]
  fn test_non_ascii() {
    let name = Domain::try_from("测试.com.").unwrap();
    assert_eq!("xn--0zwm56d.com", name.as_str());
    assert_eq!("xn--0zwm56d.com.", name.fqdn_str());

    let name = Domain::try_from("测试.中国").unwrap();
    assert_eq!("xn--0zwm56d.xn--fiqs8s", name.as_str());
    assert_eq!("xn--0zwm56d.xn--fiqs8s.", name.fqdn_str());
  }
}
