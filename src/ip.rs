use pnet::packet::ethernet::{EtherType, EtherTypes};
use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};

#[derive(Debug)]
pub enum EthernetType {
    Ipv4,
    Ipv6,
    Arp,
}

impl From<EtherType> for EthernetType {
    fn from(eth_type: EtherType) -> Self {
        match eth_type {
            EtherTypes::Ipv4 => EthernetType::Ipv4,
            EtherTypes::Ipv6 => EthernetType::Ipv6,
            EtherTypes::Arp => EthernetType::Arp,
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
    NA,
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
