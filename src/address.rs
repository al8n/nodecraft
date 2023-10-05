use core::fmt::Display;

use crate::Transformable;

mod impls;
#[cfg(feature = "dns")]
pub(crate) use impls::Kind;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use impls::{Address, AddressError, ParseAddressError};

/// Node address
pub trait NodeAddress: Clone + Eq + Display + Transformable {}
