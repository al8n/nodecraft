use core::fmt::{Debug, Display};

mod impls;
use cheap_clone::CheapClone;
#[cfg(feature = "resolver")]
pub(crate) use impls::Domain;
#[cfg(feature = "resolver")]
pub(crate) use impls::Kind;
#[cfg(feature = "std")]
pub use impls::{NodeAddress, NodeAddressError, ParseNodeAddressError};

#[cfg(feature = "transformable")]
/// Address abstraction for distributed systems
pub trait Address:
  CheapClone
  + Eq
  + core::hash::Hash
  + Debug
  + Display
  + transformable::Transformable
  + Sized
  + Unpin
  + 'static
{
}

#[cfg(not(feature = "transformable"))]
/// Address abstraction for distributed systems
pub trait Address:
  CheapClone + Eq + core::hash::Hash + Debug + Display + Sized + Unpin + 'static
{
}
