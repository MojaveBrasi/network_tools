use core::time;
use derivative::Derivative;
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, NetworkInterface, interfaces};
use pnet::packet::ethernet::EtherTypes::{Ipv4, Ipv6};
use pnet::packet::ethernet::*;
use pnet::packet::ip::IpNextHeaderProtocol;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::{self, Ipv6Packet};
use pnet::packet::{self, Packet, ipv4};
use std::time::SystemTime;
use thiserror::Error;

#[derive(Debug)]
pub enum InternetProtocol {
    Ipv4,
    Ipv6,
}

#[derive(Debug)]
pub enum TransportProtocol {
    Tcp,
    Udp,
    Icmp,
    IcmpV6,
    Unknown(u8), // preserve the raw value for unhandled cases
}

//TODO: Impl FROM pnet::EtherType -> pkt_cap::InternetProtocol
//TODO: Impl FROM pnet::NextHeaderProtocol -> pkt_cap::TransportProtocol

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("Unsupported EtherType: {0:?}")]
    UnsupportedProtocol(EtherType),

    #[error("Malformed IPv4 packet")]
    MalformedIpv4,

    #[error("Malformed IPv6 packet")]
    MalformedIpv6,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Capture {
    timestamp: SystemTime,
    length: u16,
    internet_protocol: EtherType,
    transport_protocol: IpNextHeaderProtocol,
    #[derivative(Debug = "ignore")]
    payload: Vec<u8>, // more stuff later.
}

pub fn list_interfaces() {
    let active_interfaces: Vec<datalink::NetworkInterface> = datalink::interfaces()
        .into_iter()
        .filter(|iface| iface.is_up() && !iface.ips.is_empty())
        .collect();

    for iface in &active_interfaces {
        let ipaddr = iface
            .ips
            .iter()
            .map(|ip| ip.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "Interface {}: {} | IP: {:?}, MAC: {:?}",
            iface.index, iface.name, ipaddr, iface.mac
        );
    }
}

pub fn get_interface(input: &str) -> Option<NetworkInterface> {
    let ivec = interfaces();
    let iface = ivec.into_iter().find(|iface| iface.name == input);
    iface
}

fn parse_payload(eth_pkt: &EthernetPacket) -> Result<Capture, CaptureError> {
    match eth_pkt.get_ethertype() {
        EtherTypes::Ipv4 => {
            let ipv4 = Ipv4Packet::new(&eth_pkt.payload()).ok_or(CaptureError::MalformedIpv4)?;
            // DO STUFF
            let total_len = ipv4.get_total_length() as u16;
            let header_len = ipv4.get_header_length() as u16;
            let payload_len = total_len - header_len;
            Ok(Capture {
                timestamp: SystemTime::now(),
                length: payload_len,
                internet_protocol: Ipv4,
                transport_protocol: ipv4.get_next_level_protocol(),
                payload: ipv4.payload().to_vec(),
            })
        }
        EtherTypes::Ipv6 => {
            let ipv6 = Ipv6Packet::new(&eth_pkt.payload()).ok_or(CaptureError::MalformedIpv6)?;
            Ok(Capture {
                timestamp: SystemTime::now(),
                length: ipv6.get_payload_length(),
                internet_protocol: Ipv6,
                transport_protocol: ipv6.get_next_header(),
                payload: ipv6.payload().to_vec(),
            })
        }
        other => Err(CaptureError::UnsupportedProtocol(other)),
    }
}

pub fn bind_and_listen(i: &NetworkInterface) {
    let (mut _tx, mut rx) = match datalink::channel(&i, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled Channel Type"),
        Err(e) => panic!("Error binding to interface: {}", e),
    };
    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(eth_packet) = EthernetPacket::new(&packet) {
                    let cap = parse_payload(&eth_packet);
                    println!("{:#?}", cap);
                }
            }
            Err(e) => {
                panic!("An error occured while reading: {}", e);
            }
        }
    }
}
