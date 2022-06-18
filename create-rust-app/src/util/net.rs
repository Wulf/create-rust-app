use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, TcpListener, ToSocketAddrs};

fn test_bind<A: ToSocketAddrs>(addr: A) -> bool {
    match TcpListener::bind(addr).map(|t| t.local_addr().is_ok()) {
        Ok(result) => result,
        Err(_) => false
    }
}

pub fn is_port_free(port: u16) -> bool {
    let ipv4 = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);
    let ipv6 = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, port, 0, 0);

    test_bind(ipv6) && test_bind(ipv4)
}
