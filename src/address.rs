use core::fmt::{Debug, Display};

mod impls;
use cheap_clone::CheapClone;

#[cfg(any(feature = "std", feature = "alloc"))]
pub use impls::{Domain, DomainRef, HostAddr, HostAddrRef, ParseDomainError, ParseHostAddrError};

/// Address abstraction for distributed systems
pub trait Address:
  CheapClone + Eq + Ord + core::hash::Hash + Debug + Display + Sized + Unpin + 'static
{
}

impl<T> Address for T where
  T: CheapClone + Eq + Ord + core::hash::Hash + Debug + Display + Sized + Unpin + 'static
{
}
