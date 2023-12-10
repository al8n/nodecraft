use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6, IpAddr, Ipv4Addr, Ipv6Addr};

use crate::Id;

impl Id for SocketAddr {}
impl Id for SocketAddrV4 {}
impl Id for SocketAddrV6 {}
impl Id for IpAddr {}
impl Id for Ipv4Addr {}
impl Id for Ipv6Addr {}

