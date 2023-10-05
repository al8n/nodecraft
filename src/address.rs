use core::fmt::Display;

use crate::Transformable;

mod impls;
#[cfg(feature = "resolver")]
pub(crate) use impls::Kind;
#[cfg(feature = "std")]
pub use impls::{Address, AddressError, ParseAddressError};

/// Node address
pub trait NodeAddress: Clone + Eq + Display + Transformable {}
