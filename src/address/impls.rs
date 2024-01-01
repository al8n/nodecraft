#[cfg(feature = "std")]
mod address;
#[cfg(feature = "resolver")]
pub(crate) use address::DnsName;
#[cfg(feature = "std")]
pub use address::*;

#[cfg(feature = "std")]
mod socket_addr;
