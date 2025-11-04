use idna::uts46::{
  AsciiDenyList, DnsLength, ErrorPolicy, Hyphens, ProcessingSuccess, Uts46, verify_dns_length,
};

use super::{Domain, ParseDomainError};

/// A reference to a [`Domain`] that is guaranteed to be valid.
#[derive(Debug, Clone, Copy, Eq)]
pub struct DomainRef<'a> {
  data: &'a str,
  fqdn: bool,
  idn: bool,
}

impl PartialEq for DomainRef<'_> {
  fn eq(&self, other: &Self) -> bool {
    self.as_str().eq(other.as_str())
  }
}

impl PartialOrd for DomainRef<'_> {
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl core::hash::Hash for DomainRef<'_> {
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    self.as_str().hash(state)
  }
}

impl core::borrow::Borrow<str> for DomainRef<'_> {
  fn borrow(&self) -> &str {
    self.as_str()
  }
}

impl Ord for DomainRef<'_> {
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    self.as_str().cmp(other.as_str())
  }
}

impl<'a> DomainRef<'a> {
  #[inline]
  fn new(s: &'a str, idn: bool) -> Self {
    let fqdn = s.ends_with('.');
    Self { data: s, fqdn, idn }
  }

  /// Returns the domain as a str slice.
  #[inline]
  pub fn as_str(&self) -> &'a str {
    self.data.trim_end_matches('.')
  }

  /// Returns the original str used to create this [`DomainRef`].
  #[inline]
  pub fn as_source_str(&self) -> &'a str {
    self.data
  }

  /// Returns `true` if the domain is a fully qualified domain name.
  #[inline]
  pub const fn is_fqdn(&self) -> bool {
    self.fqdn
  }

  /// Returns `true` if the domain is an internationalized domain name.
  #[inline]
  pub const fn is_idn(&self) -> bool {
    self.idn
  }

  /// Returns the owned version of the domain.
  pub fn to_owned(self) -> Domain {
    match (self.fqdn, self.idn) {
      (true, true) => {
        let res = Uts46::new()
          .to_ascii(
            self.data.as_bytes(),
            AsciiDenyList::EMPTY,
            Hyphens::Allow,
            DnsLength::VerifyAllowRootDot,
          )
          .expect("DomainRef must be valid a internationalized domain name");
        Domain(smol_str_0_3::SmolStr::from(res))
      }
      (true, false) => Domain(self.data.into()),
      (false, true) => {
        let res = Uts46::new()
          .to_ascii(
            self.data.as_bytes(),
            AsciiDenyList::EMPTY,
            Hyphens::Allow,
            DnsLength::VerifyAllowRootDot,
          )
          .expect("DomainRef must be valid a internationalized domain name");
        Domain(smol_str_0_3::format_smolstr!("{}.", res))
      }
      (false, false) => Domain(smol_str_0_3::format_smolstr!("{}.", self.data)),
    }
  }
}

impl<'a> TryFrom<&'a str> for DomainRef<'a> {
  type Error = ParseDomainError;

  fn try_from(domain: &'a str) -> Result<Self, ParseDomainError> {
    DomainRef::try_from(domain.as_bytes())
  }
}

struct Sinker {
  buf: [u8; 254], // one more for the trailing dot
  pos: usize,
}

impl Sinker {
  #[inline]
  const fn new() -> Self {
    Self {
      buf: [0; 254],
      pos: 0,
    }
  }

  #[inline]
  fn as_str(&self) -> &str {
    core::str::from_utf8(&self.buf[0..self.pos]).expect("valid utf-8")
  }
}

impl core::fmt::Write for Sinker {
  fn write_str(&mut self, s: &str) -> core::fmt::Result {
    let len = s.len();
    if self.pos + len > 254 {
      return Err(core::fmt::Error);
    }
    self.buf[self.pos..self.pos + len].copy_from_slice(s.as_bytes());
    self.pos += len;
    Ok(())
  }
}

impl<'a> TryFrom<&'a [u8]> for DomainRef<'a> {
  type Error = ParseDomainError;

  fn try_from(domain: &'a [u8]) -> Result<Self, ParseDomainError> {
    // fast path
    if domain.is_ascii() {
      return super::validate(domain).map(|_| {
        let domain = core::str::from_utf8(domain).expect("ASCII must be valid utf-8");
        Self::new(domain, false)
      });
    }

    let uts46 = Uts46::new();
    let mut sink = Sinker::new();
    let result = uts46.process(
      domain,
      AsciiDenyList::URL,
      Hyphens::Allow,
      ErrorPolicy::FailFast,
      |_, _, _| false, // Force ToASCII processing
      &mut sink,
      None,
    );

    let ascii_str = core::str::from_utf8(domain).map_err(|_| ParseDomainError)?;
    Ok(match result {
      Ok(res) => match res {
        ProcessingSuccess::WroteToSink => {
          let s = sink.as_str();
          if !verify_dns_length(s, true) {
            return Err(ParseDomainError);
          }

          Self::new(ascii_str, true)
        }
        _ => unreachable!("ASCII domain should already be processed by fast path"),
      },
      Err(_) => return Err(ParseDomainError),
    })
  }
}

#[cfg(feature = "serde")]
const _: () = {
  impl serde::Serialize for DomainRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: serde::Serializer,
    {
      self.as_str().serialize(serializer)
    }
  }

  impl<'de> serde::Deserialize<'de> for DomainRef<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: serde::Deserializer<'de>,
    {
      let s = <&'de str>::deserialize(deserializer)?;
      s.try_into().map_err(serde::de::Error::custom)
    }
  }
};

#[cfg(all(any(feature = "std", feature = "alloc"), test))]
mod tests {
  use super::*;

  #[cfg(any(feature = "alloc", feature = "std"))]
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
    (
      "twohundredandfiftythreecharacters.twohundredandfiftythreecharacters.twohundredandfiftythreecharacters.twohundredandfiftythreecharacters.twohundredandfiftythreecharacters.twohundredandfiftythreecharacters.twohundredandfiftythreecharacters.twohundredandfi",
      true,
    ),
    (
      "twohundredandfiftyfourcharacters.twohundredandfiftyfourcharacters.twohundredandfiftyfourcharacters.twohundredandfiftyfourcharacters.twohundredandfiftyfourcharacters.twohundredandfiftyfourcharacters.twohundredandfiftyfourcharacters.twohundredandfiftyfourc",
      false,
    ),
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
      let name_ref = DomainRef::try_from(*input);
      assert_eq!(*expected, name_ref.is_ok());
    }
  }

  #[cfg(any(feature = "std", feature = "alloc"))]
  #[test]
  fn test_borrow() {
    use std::collections::HashSet;
    let name = DomainRef::try_from("localhost.").unwrap();
    let mut set = HashSet::new();
    set.insert(name);

    assert!(set.contains("localhost"));
  }

  #[test]
  fn test_try_from_bytes() {
    use super::DomainRef;

    let name = DomainRef::try_from(b"localhost".as_slice()).unwrap();
    assert_eq!("localhost", name.as_str());
    assert!(!name.is_fqdn());
    let name = name.to_owned();
    assert_eq!("localhost", name.as_str());
    assert_eq!("localhost.", name.fqdn_str());

    let name = DomainRef::try_from(b"localhost.".as_slice()).unwrap();
    assert_eq!("localhost.", name.as_source_str());
    assert!(name.is_fqdn());
    assert!(!name.is_idn());

    let name = DomainRef::try_from(b"labelendswithnumber1.bar.com".as_slice()).unwrap();
    assert_eq!(name.as_str(), "labelendswithnumber1.bar.com");

    let name = DomainRef::try_from(b"labelendswithnumber1.bar.com.".as_slice()).unwrap();
    assert_eq!(name.as_source_str(), "labelendswithnumber1.bar.com.");
  }

  #[test]
  fn test_try_from_str() {
    use super::DomainRef;

    let name = DomainRef::try_from("localhost").unwrap();
    assert_eq!("localhost", name.as_str());
    assert!(!name.is_fqdn());
    let name = name.to_owned();
    assert_eq!("localhost", name.as_str());
    assert_eq!("localhost.", name.fqdn_str());

    let name = DomainRef::try_from("localhost.").unwrap();
    assert_eq!("localhost", name.as_str());
    assert_eq!("localhost.", name.as_source_str());
    assert!(name.is_fqdn());
    let name = name.to_owned();
    assert_eq!("localhost", name.as_str());
    assert_eq!("localhost.", name.fqdn_str());

    let name = DomainRef::try_from("labelendswithnumber1.bar.com").unwrap();
    assert_eq!(name.as_str(), "labelendswithnumber1.bar.com");

    let name = DomainRef::try_from("labelendswithnumber1.bar.com.").unwrap();
    assert_eq!(name.as_source_str(), "labelendswithnumber1.bar.com.");
  }

  #[test]
  fn test_eq_and_ord() {
    let name1 = DomainRef::try_from("localhost").unwrap();
    let name2 = DomainRef::try_from("localhost.").unwrap();
    assert_eq!(name1, name2);
    assert_eq!(name1.as_str(), name2.as_str());
    assert_ne!(name1.as_source_str(), name2.as_source_str());

    assert!(name1.partial_cmp(&name2) == Some(core::cmp::Ordering::Equal));
  }

  #[test]
  fn test_non_ascii() {
    let name = DomainRef::try_from("测试.com.").unwrap();
    assert_eq!("测试.com.", name.as_source_str());
    assert!(name.is_fqdn());
    assert!(name.is_idn());
    let name = name.to_owned();
    assert_eq!("xn--0zwm56d.com", name.as_str());
    assert_eq!("xn--0zwm56d.com.", name.fqdn_str());

    let name = DomainRef::try_from("测试.中国").unwrap();
    assert_eq!("测试.中国", name.as_str());
    let name = name.to_owned();
    assert_eq!("xn--0zwm56d.xn--fiqs8s", name.as_str());
    assert_eq!("xn--0zwm56d.xn--fiqs8s.", name.fqdn_str());
  }
}
