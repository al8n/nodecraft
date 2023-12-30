use core::fmt::{Debug, Display};

use transformable::Transformable;

mod impls;
use cheap_clone::CheapClone;
#[cfg(feature = "resolver")]
pub(crate) use impls::Kind;
#[cfg(feature = "std")]
pub use impls::{NodeAddress, NodeAddressError, ParseNodeAddressError};

/// Address abstraction for distributed systems
pub trait Address:
  CheapClone + Eq + core::hash::Hash + Debug + Display + Transformable + Sized + Unpin + 'static
{
}
