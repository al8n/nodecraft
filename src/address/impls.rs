#[cfg(feature = "std")]
mod address;
#[cfg(feature = "resolver")]
pub(crate) use address::Domain;
#[cfg(feature = "std")]
pub use address::*;

#[cfg(feature = "std")]
mod socket_addr;
