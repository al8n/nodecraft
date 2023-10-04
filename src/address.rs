use core::fmt::Display;

use crate::Transformable;

mod impls;

/// Node address
pub trait NodeAddress: Clone + Eq + Display + Transformable {}
