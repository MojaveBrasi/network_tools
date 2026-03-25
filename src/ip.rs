use pnet::packet::ethernet::EtherTypes::{Ipv4, Ipv6};
use pnet::packet::ethernet::{EtherType, EtherTypes};
use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};
#[derive(Debug)]
pub enum InternetProtocol {
    Ipv4,
    Ipv6,
}

impl From<EtherType> for InternetProtocol {
    fn from(eth_type: EtherType) -> Self {
        match eth_type {
            EtherTypes::Ipv4 => InternetProtocol::Ipv4,
            EtherTypes::Ipv6 => InternetProtocol::Ipv6,
            other => panic!("Ethertype not Ipv4 or Ipv6. Cannot convert {:?}", other),
        }
    }
}

#[derive(Debug)]
pub enum TransportProtocol {
    Tcp,
    Udp,
    Icmp,
    IcmpV6,
    Unknown(u8), // preserve the raw value for unhandled cases
}

impl From<IpNextHeaderProtocol> for TransportProtocol {
    fn from(proto: IpNextHeaderProtocol) -> Self {
        match proto {
            IpNextHeaderProtocols::Tcp => TransportProtocol::Tcp,
            IpNextHeaderProtocols::Udp => TransportProtocol::Udp,
            IpNextHeaderProtocols::Icmp => TransportProtocol::Icmp,
            IpNextHeaderProtocols::Icmpv6 => TransportProtocol::IcmpV6,
            other => panic!("Transport protocol not suppported: {:?}", other),
        }
    }
}
