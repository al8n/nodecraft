use core::{
  fmt::Display,
  hash::Hash,
};

use crate::Transformable;

mod impls;

/// Node id
pub trait NodeId: Clone + Eq + Hash + Display + Transformable { }
