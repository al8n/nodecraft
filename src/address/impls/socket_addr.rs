use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};

use crate::Address;

impl Address for SocketAddr {}

impl Address for SocketAddrV4 {}

impl Address for SocketAddrV6 {}
