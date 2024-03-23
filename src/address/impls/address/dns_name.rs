use core::{fmt, hash::Hash};

use smol_str::SmolStr;

/// A type which encapsulates a string (borrowed or owned) that is a syntactically valid DNS name.
#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(
  feature = "rkyv",
  derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "rkyv", archive(check_bytes, compare(PartialEq)))]
#[cfg_attr(
  feature = "rkyv",
  archive_attr(derive(PartialEq, Eq, PartialOrd, Ord, Hash), repr(transparent))
)]
pub(crate) struct DnsName(SmolStr);

impl core::fmt::Display for DnsName {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.as_str().fmt(f)
  }
}

impl core::borrow::Borrow<str> for DnsName {
  fn borrow(&self) -> &str {
    self.as_str()
  }
}

impl core::borrow::Borrow<SmolStr> for DnsName {
  fn borrow(&self) -> &SmolStr {
    &self.0
  }
}

impl DnsName {
  /// Returns the str representation.
  #[inline]
  pub fn as_str(&self) -> &str {
    self.0.trim_end_matches('.')
  }

  pub fn terminate_str(&self) -> &str {
    self.0.as_str()
  }
}

#[cfg(feature = "alloc")]
impl TryFrom<String> for DnsName {
  type Error = InvalidDnsNameError;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    validate(value.as_bytes())?;
    if value.len() < 23 {
      if value.ends_with('.') {
        let mut buf = [0u8; 23];
        buf[..value.len()].copy_from_slice(value.as_bytes());
        buf[value.len()] = b'.';
        Ok(Self(
          core::str::from_utf8(&buf[..value.len() + 1])
            .unwrap()
            .into(),
        ))
      } else {
        Ok(Self(value.into()))
      }
    } else if !value.ends_with('.') {
      Ok(Self(format!("{}.", value).into()))
    } else {
      Ok(Self(value.into()))
    }
  }
}

#[cfg(feature = "alloc")]
impl TryFrom<&String> for DnsName {
  type Error = InvalidDnsNameError;

  fn try_from(value: &String) -> Result<Self, Self::Error> {
    value.as_str().try_into()
  }
}

impl<'a> TryFrom<&'a str> for DnsName {
  type Error = InvalidDnsNameError;

  fn try_from(value: &'a str) -> Result<Self, Self::Error> {
    validate(value.as_bytes())?;
    if value.len() < 23 {
      if value.ends_with('.') {
        let mut buf = [0u8; 23];
        buf[..value.len()].copy_from_slice(value.as_bytes());
        buf[value.len()] = b'.';
        Ok(Self(
          core::str::from_utf8(&buf[..value.len() + 1])
            .unwrap()
            .into(),
        ))
      } else {
        Ok(Self(value.into()))
      }
    } else if !value.ends_with('.') {
      Ok(Self(format!("{}.", value).into()))
    } else {
      Ok(Self(value.into()))
    }
  }
}

impl<'a> TryFrom<&'a [u8]> for DnsName {
  type Error = InvalidDnsNameError;

  fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
    validate(value)?;
    if value.len() < 23 {
      if value.ends_with(&[b'.']) {
        let mut buf = [0u8; 23];
        buf[..value.len()].copy_from_slice(value);
        buf[value.len()] = b'.';
        Ok(Self(
          core::str::from_utf8(&buf[..value.len() + 1])
            .unwrap()
            .into(),
        ))
      } else {
        Ok(Self(core::str::from_utf8(value).unwrap().into()))
      }
    } else if !value.ends_with(&[b'.']) {
      Ok(Self(
        format!("{}.", core::str::from_utf8(value).unwrap()).into(),
      ))
    } else {
      Ok(Self(core::str::from_utf8(value).unwrap().into()))
    }
  }
}

impl AsRef<str> for DnsName {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

/// The provided input could not be parsed because
/// it is not a syntactically-valid DNS Name.
#[derive(Debug)]
pub struct InvalidDnsNameError;

impl fmt::Display for InvalidDnsNameError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str("invalid dns name")
  }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidDnsNameError {}

fn validate(input: &[u8]) -> Result<(), InvalidDnsNameError> {
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

  if input.len() > MAX_NAME_LENGTH {
    return Err(InvalidDnsNameError);
  }

  for ch in input {
    state = match (state, ch) {
      (Start | Next | NextAfterNumericOnly | Hyphen { .. }, b'.') => {
        return Err(InvalidDnsNameError)
      }
      (Subsequent { .. }, b'.') => Next,
      (NumericOnly { .. }, b'.') => NextAfterNumericOnly,
      (Subsequent { len } | NumericOnly { len } | Hyphen { len }, _) if len >= MAX_LABEL_LENGTH => {
        return Err(InvalidDnsNameError)
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
      _ => return Err(InvalidDnsNameError),
    };
  }

  if matches!(
    state,
    Start | Hyphen { .. } | NumericOnly { .. } | NextAfterNumericOnly
  ) {
    return Err(InvalidDnsNameError);
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
      let name_ref = DnsName::try_from(*input);
      assert_eq!(*expected, name_ref.is_ok());
      let name = DnsName::try_from(input.to_string());
      assert_eq!(*expected, name.is_ok());
    }
  }

  #[cfg(feature = "alloc")]
  #[test]
  fn test_basic() {
    let name = DnsName::try_from(&"localhost".to_string()).unwrap();
    assert_eq!("localhost", name.as_ref());
    let err = InvalidDnsNameError;
    println!("{}", err);
  }

  #[cfg(feature = "std")]
  #[test]
  fn test_borrow() {
    use std::collections::HashSet;
    let name = DnsName::try_from("localhost").unwrap();
    let mut set = HashSet::new();
    set.insert(name);

    assert!(set.contains("localhost"));
    assert!(set.contains(&SmolStr::from("localhost")));
  }

  #[test]
  fn test_try_from_bytes() {
    use super::DnsName;

    let name = DnsName::try_from(b"localhost".as_slice()).unwrap();
    assert_eq!("localhost", name.as_str());

    let name = DnsName::try_from(b"localhost.".as_slice()).unwrap();
    assert_eq!("localhost", name.as_str());

    let name = DnsName::try_from(b"labelendswithnumber1.bar.com".as_slice()).unwrap();
    assert_eq!(name.to_string().as_str(), "labelendswithnumber1.bar.com");

    let name = DnsName::try_from(b"labelendswithnumber1.bar.com.".as_slice()).unwrap();
    assert_eq!(name.to_string().as_str(), "labelendswithnumber1.bar.com");
  }
}
