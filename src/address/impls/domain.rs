use super::{ParseDomainError, validate};
use std::string::String;

use idna::{
  AsciiDenyList,
  uts46::{DnsLength, Hyphens, Uts46},
};
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
pub struct Domain(pub(crate) SmolStr);

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
    if domain.is_ascii() {
      validate(domain)?;

      let domain = core::str::from_utf8(domain).expect("bytes must be valid utf8");
      // Early return if already has trailing dot
      if domain.ends_with('.') {
        return Ok(Self(domain.into()));
      }

      Ok(Domain(smol_str03::format_smolstr!("{}.", domain)))
    } else {
      let valid_domain = Uts46::new()
        .to_ascii(
          domain,
          AsciiDenyList::URL,
          Hyphens::Allow,
          DnsLength::VerifyAllowRootDot,
        )
        .map_err(|_| ParseDomainError)?;

      if valid_domain.ends_with('.') {
        return Ok(Self(valid_domain.into()));
      }

      Ok(Self(smol_str03::format_smolstr!("{}.", valid_domain)))
    }
  }
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl TryFrom<String> for Domain {
  type Error = ParseDomainError;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::try_from_inner(value.as_bytes())
  }
}

#[cfg(any(feature = "alloc", feature = "std"))]
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

#[cfg(feature = "arbitrary")]
const _: () = {
  use arbitrary::{Arbitrary, Result, Unstructured};

  impl<'a> Arbitrary<'a> for Domain {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
      // Generate between 1 and 4 labels
      let label_count = u.int_in_range(1..=4)?;
      let mut domain = String::new();

      for i in 0..label_count {
        if i > 0 {
          domain.push('.');
        }

        // Generate label length (1-63)
        let len = u.int_in_range(1..=63)?;

        // First character can't be hyphen but can be underscore
        let first_char = if u.arbitrary::<bool>()? {
          // letter
          let c = u.int_in_range(0..=51)?;
          if c < 26 { b'a' + c } else { b'A' + (c - 26) }
        } else if u.arbitrary::<bool>()? {
          // number
          u.int_in_range(b'0'..=b'9')?
        } else {
          b'_'
        } as char;

        domain.push(first_char);

        // Rest of the characters
        for _ in 1..len {
          let c = match u.int_in_range(0..=4)? {
            0 => u.int_in_range(b'a'..=b'z')? as char,
            1 => u.int_in_range(b'A'..=b'Z')? as char,
            2 => u.int_in_range(b'0'..=b'9')? as char,
            3 => '_',
            _ => {
              if len > 1 {
                '-'
              } else {
                u.int_in_range(b'a'..=b'z')? as char
              }
            }
          };
          domain.push(c);
        }

        // Ensure last char isn't hyphen
        if domain.ends_with('-') {
          domain.push('a');
        }
      }

      // Ensure last label isn't numeric-only
      if domain
        .split('.')
        .next_back()
        .unwrap()
        .chars()
        .all(|c| c.is_ascii_digit())
      {
        domain.push('x');
      }

      // Optionally add trailing dot
      if u.arbitrary::<bool>()? {
        domain.push('.');
      }

      Domain::try_from(domain).map_err(|_| arbitrary::Error::IncorrectFormat)
    }
  }
};

#[cfg(feature = "quickcheck")]
const _: () = {
  use quickcheck::{Arbitrary, Gen};

  impl Arbitrary for Domain {
    fn arbitrary(g: &mut Gen) -> Self {
      let size = (usize::arbitrary(g) % 3) + 1; // 1-3 labels

      let mut domain = String::new();
      for i in 0..size {
        if i > 0 {
          domain.push('.');
        }

        let len = (usize::arbitrary(g) % 63) + 1;
        let chars: String = std::iter::from_fn(|| {
          let r = usize::arbitrary(g) % 64;
          Some(match r {
            0..=25 => (b'a' + (r as u8)) as char,
            26..=51 => (b'A' + (r as u8 - 26)) as char,
            52..=61 => (b'0' + (r as u8 - 52)) as char,
            62 => '_',
            _ => '-',
          })
        })
        .take(len)
        .collect();

        // Ensure valid first/last chars
        let mut label = chars;
        if label.starts_with('-') {
          label.replace_range(0..1, "a");
        }
        if label.ends_with('-') {
          label.replace_range(label.len() - 1..label.len(), "a");
        }

        domain.push_str(&label);
      }

      // Ensure last label isn't numeric-only
      if domain
        .split('.')
        .next_back()
        .unwrap()
        .chars()
        .all(|c| c.is_ascii_digit())
      {
        domain.push('x');
      }

      // Maybe add trailing dot
      if bool::arbitrary(g) {
        domain.push('.');
      }

      Domain::try_from(domain).unwrap_or_else(|_| Domain::try_from("example.com").unwrap())
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
      let s = self.as_str().to_string();
      Box::new(
        s.shrink()
          .filter(|s| !s.is_empty())
          .filter_map(|s| Domain::try_from(s).ok()),
      )
    }
  }
};

#[cfg(all(any(feature = "std", feature = "alloc"), test))]
mod tests {
  use core::str::FromStr;

  use super::*;

  #[cfg(any(feature = "alloc", feature = "std"))]
  static TESTS: &[(&str, bool)] = &[
    ("", false),
    (".", true),
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
    ("-example.com", false),
    ("example-.com", false),
    ("1.2.3.4.com", true),
    ("123.numeric-only-first-label", true),
    ("a123b.com", true),
    ("numeric-only-middle-label.4.com", true),
    ("1000-sans.badssl.com", true),
    (
      "twohundredandfiftythreecharacters.twohundredandfiftythreecharacters.twohundredandfiftythreecharacters.twohundredandfiftythreecharacters.twohundredandfiftythreecharacters.twohundredandfiftythreecharacters.twohundredandfiftythreecharacters.twohundredandfi",
      true,
    ),
    ("123.", false),
    (
      "twohundredandfiftyfourcharacters.twohundredandfiftyfourcharacters.twohundredandfiftyfourcharacters.twohundredandfiftyfourcharacters.twohundredandfiftyfourcharacters.twohundredandfiftyfourcharacters.twohundredandfiftyfourcharacters.twohundredandfiftyfourc",
      false,
    ),
    ("abc@abc.com", false),
    ("测试.com", true),
    ("测试.中国", true),
    ("测试@测试.中国", false),
  ];

  #[cfg(any(feature = "alloc", feature = "std"))]
  #[test]
  fn test_validation() {
    for (input, expected) in TESTS {
      #[cfg(any(feature = "std", feature = "alloc"))]
      println!("test: {:?} expected valid? {:?}", input, expected);
      let name_ref = Domain::try_from(*input);
      assert_eq!(*expected, name_ref.is_ok());
      let name = Domain::try_from(input.to_string());
      assert_eq!(*expected, name.is_ok());
    }
  }

  #[cfg(any(feature = "alloc", feature = "std"))]
  #[test]
  fn test_basic() {
    let name = Domain::try_from(&"localhost".to_string()).unwrap();
    assert_eq!("localhost", name.as_ref());
    let err = ParseDomainError;
    println!("{}", err);
  }

  #[cfg(any(feature = "std", feature = "alloc"))]
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
    assert_eq!(name.as_str(), "labelendswithnumber1.bar.com");

    let name = Domain::try_from(b"labelendswithnumber1.bar.com.".as_slice()).unwrap();
    assert_eq!(name.as_str(), "labelendswithnumber1.bar.com");
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
    assert_eq!(name.as_str(), "labelendswithnumber1.bar.com");

    let name = Domain::try_from("labelendswithnumber1.bar.com.").unwrap();
    assert_eq!(name.as_str(), "labelendswithnumber1.bar.com");
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

  #[cfg(feature = "serde")]
  #[quickcheck_macros::quickcheck]
  fn fuzzy_serde(node: Domain) -> bool {
    let serialized = serde_json::to_string(&node).unwrap();
    let deserialized: Domain = serde_json::from_str(&serialized).unwrap();
    node == deserialized
  }
}
