use core::fmt::Display;

use crate::Transformable;

mod impls;
#[cfg(feature = "resolver")]
pub(crate) use impls::Kind;
#[cfg(feature = "std")]
pub use impls::{NodeAddress, NodeAddressError, ParseNodeAddressError};

/// Address abstraction for distributed systems
pub trait Address:
  Clone + Eq + core::hash::Hash + Display + Transformable + Sized + 'static
{
}
