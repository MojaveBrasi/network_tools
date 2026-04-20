use derivative::Derivative;
use pnet::datalink::NetworkInterface;
use pnet::util::MacAddr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Interface {
    //TODO: Map interfaces to struct at user's discretion. Give aliases for interfaces. Store
    //interfaces in database. Add more helpful fields later.
    index: u32,
    ipv4_addr: Vec<Ipv4Addr>,
    ipv6_addr: Vec<Ipv6Addr>,
    mac_addr: Option<MacAddr>,
}

#[derive(Debug)]
struct KnownInterfaces {
    unique_iface_count: u16,
    unique_ifaces: Vec<NetworkInterface>,
}

#[derive(Debug)]
struct KnownAddresses {
    unique_addr_count: u16,
    unique_addrs: Vec<IpRecord>,
}

#[derive(Debug)]
struct IpRecord {
    mac: MacAddr,
    ipv4: Option<Ipv4Addr>,
    ipv6: Option<Ipv6Addr>,
}
pub fn ip_to_bytes(addr: IpAddr) -> [u8; 16] {
    match addr {
        IpAddr::V4(v4) => v4.to_ipv6_mapped().octets(),
        IpAddr::V6(v6) => v6.octets(),
    }
}

pub fn bytes_to_ip(bytes: &[u8; 16]) -> IpAddr {
    let v6 = Ipv6Addr::from(*bytes);
    // Try to "unmap" back to IPv4 if applicable
    if let Some(v4) = v6.to_ipv4_mapped() {
        IpAddr::V4(v4)
    } else {
        IpAddr::V6(v6)
    }
}
