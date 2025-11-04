use core::fmt::{Debug, Display};

use cheap_clone::CheapClone;

/// Host address type alias
#[cfg(feature = "hostaddr")]
#[cfg_attr(docsrs, doc(cfg(feature = "hostaddr")))]
pub type HostAddr = hostaddr::HostAddr<smol_str_0_3::SmolStr>;

/// Domain type alias
#[cfg(feature = "hostaddr")]
#[cfg_attr(docsrs, doc(cfg(feature = "hostaddr")))]
pub type Domain = hostaddr::Domain<smol_str_0_3::SmolStr>;

/// Address abstraction for distributed systems
pub trait Address:
  CheapClone + Eq + Ord + core::hash::Hash + Debug + Display + Sized + Unpin + 'static
{
}

impl<T> Address for T where
  T: CheapClone + Eq + Ord + core::hash::Hash + Debug + Display + Sized + Unpin + 'static
{
}
