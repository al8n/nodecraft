use core::fmt::{Debug, Display};

mod impls;
use cheap_clone::CheapClone;
#[cfg(feature = "resolver")]
pub(crate) use impls::Domain;
#[cfg(feature = "resolver")]
pub(crate) use impls::Kind;
#[cfg(feature = "std")]
pub use impls::{NodeAddress, ParseNodeAddressError};

/// Address abstraction for distributed systems
pub trait Address:
  CheapClone + Eq + Ord + core::hash::Hash + Debug + Display + Sized + Unpin + 'static
{
}

impl<T> Address for T where
  T: CheapClone + Eq + Ord + core::hash::Hash + Debug + Display + Sized + Unpin + 'static
{
}
