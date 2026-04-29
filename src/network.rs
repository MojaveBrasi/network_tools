use chrono::{DateTime, Utc};
use derivative::Derivative;
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, NetworkInterface};
use pnet::ipnetwork::IpNetwork;
use pnet::packet::Packet;
use pnet::packet::arp::*;
use pnet::packet::ethernet::*;
use pnet::packet::ip::IpNextHeaderProtocol;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::util::MacAddr;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use thiserror::Error;
use tokio::sync::mpsc;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct IpCapture {
    pub timestamp: DateTime<Utc>,
    pub source: IpAddr,
    pub dest: IpAddr,
    pub ethernet_frame_type: EtherType,
    pub transport_protocol: IpNextHeaderProtocol,
    pub length: u16,
    #[derivative(Debug = "ignore")]
    payload: Vec<u8>, // TODO: Change to hex value. Serialize to JSON, add to DB, give user option
}

impl fmt::Display for IpCapture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}] Source: {} | EtherType: {} | Protocol: {} | Length: {} bytes",
            self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
            self.source,
            self.ethernet_frame_type,
            self.transport_protocol,
            self.length
        )
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct ArpCapture {
    timestamp: DateTime<Utc>,
    source: IpAddr,
    source_mac: MacAddr,
    operation: ArpOperation,
    hardware_type: ArpHardwareType,
}

impl fmt::Display for ArpCapture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}] Source: {} - {} | Operation: {:?} | Hardware Type : {:?}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
            self.source,
            self.source_mac,
            self.operation,
            self.hardware_type,
        )
    }
}

#[derive(Debug)]
pub enum Capture {
    IP(IpCapture),
    ARP(ArpCapture),
}

impl fmt::Display for Capture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Capture::IP(ip) => write!(f, "{}", ip),
            Capture::ARP(arp) => write!(f, "{}", arp),
        }
    }
}

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("Unsupported EtherType: {0:?}")]
    UnsupportedProtocol(EtherType),

    #[error("Malformed IPv4 packet")]
    MalformedIpv4,

    #[error("Malformed IPv6 packet")]
    MalformedIpv6,

    #[error("Malformed ARP packet")]
    MalformedArp,
}

#[derive(Debug)]
pub struct DisplayIpAddr<'a>(pub &'a Vec<IpNetwork>);

impl<'a> fmt::Display for DisplayIpAddr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatted = self
            .0
            .iter()
            .map(|ip| ip.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{}", formatted)
    }
}

#[derive(Debug)]
pub struct DisplayMacAddr(pub Option<MacAddr>);

impl fmt::Display for DisplayMacAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Some(mac) => write!(f, "{}", mac),
            None => write!(f, "N/A"),
        }
    }
}

struct Device {
    mac: Option<MacAddr>,
    ipv4: Option<Ipv4Addr>,
    ipv6: Option<Ipv6Addr>,
    //TODO: Amount of data handled in bytes
    //TODO: IP Geolocation
}
/// Every unique IP Address has a record of every other IP it has communicated with, as well as the
/// amount of data handled and the geolocation
pub struct IpRecord {
    device: Device,
    friends: Vec<Device>,
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
pub fn interface_list_local() {
    let active_interfaces: Vec<pnet::datalink::NetworkInterface> = pnet::datalink::interfaces()
        .into_iter()
        .filter(|iface| iface.is_up() && !iface.ips.is_empty())
        .collect();

    println!("------------");
    for iface in &active_interfaces {
        println!(
            "Interface {}: {} | IPAddrs: {} | MAC: {}",
            iface.index,
            iface.name,
            DisplayIpAddr(&iface.ips),
            DisplayMacAddr(iface.mac),
        );
        println!("------------");
    }
}

pub fn get_interface(input: &str) -> Option<NetworkInterface> {
    let ivec = pnet::datalink::interfaces();
    let iface = ivec.into_iter().find(|iface| iface.name == input);
    iface
}

pub fn interface_info(iface_name: &str) {
    if let Some(iface) = get_interface(&iface_name) {
        println!("--------------------");
        println!(
            "Info for interface {}: {} | IPAddrs: {} | MAC: {} | Flags: {}",
            iface.index,
            iface.name,
            DisplayIpAddr(&iface.ips),
            DisplayMacAddr(iface.mac),
            //TODO: Implementation for Displaying NetworkInterface.flags
            iface.flags
        );
        println!("--------------------");
    }
}


fn parse_payload(eth_pkt: &EthernetPacket) -> Result<Capture, CaptureError> {
    match eth_pkt.get_ethertype() {
        EtherTypes::Ipv4 => {
            let ipv4 = Ipv4Packet::new(&eth_pkt.payload()).ok_or(CaptureError::MalformedIpv4)?;
            let total_len = ipv4.get_total_length() as u16;
            let header_len = ipv4.get_header_length() as u16;
            let payload_len = total_len - header_len;
            Ok(Capture::IP(IpCapture {
                timestamp: Utc::now(),
                source: IpAddr::from(ipv4.get_source()),
                dest: IpAddr::from(ipv4.get_destination()),
                length: payload_len,
                ethernet_frame_type: EtherTypes::Ipv4,
                transport_protocol: ipv4.get_next_level_protocol(),
                payload: ipv4.payload().to_vec(),
            }))
        }
        EtherTypes::Ipv6 => {
            let ipv6 = Ipv6Packet::new(&eth_pkt.payload()).ok_or(CaptureError::MalformedIpv6)?;
            Ok(Capture::IP(IpCapture {
                timestamp: Utc::now(),
                source: IpAddr::from(ipv6.get_source()),
                dest: IpAddr::from(ipv6.get_destination()),
                length: ipv6.get_payload_length(),
                ethernet_frame_type: EtherTypes::Ipv6,
                transport_protocol: ipv6.get_next_header(),
                payload: ipv6.payload().to_vec(),
            }))
        }
        EtherTypes::Arp => {
            let arp = ArpPacket::new(&eth_pkt.payload()).ok_or(CaptureError::MalformedArp)?;
            Ok(Capture::ARP(ArpCapture {
                timestamp: Utc::now(),
                source: IpAddr::from(arp.get_sender_proto_addr()),
                source_mac: MacAddr::from(arp.get_sender_hw_addr()),
                operation: arp.get_operation(),
                hardware_type: arp.get_hardware_type(),
            }))
        }
        other => Err(CaptureError::UnsupportedProtocol(other)), //TODO: Consider other Ethertypes
    }
}

pub fn bind_and_listen(i: &NetworkInterface, sender: mpsc::Sender<IpCapture>) {
    let (mut _tx, mut eth_reciever) = match datalink::channel(&i, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled Channel Type"),
        Err(e) => panic!("Error binding to interface: {}", e),
    };
    loop {
        match eth_reciever.next() {
            Ok(packet) => {
                if let Some(eth_packet) = EthernetPacket::new(&packet) {
                    let capresult = parse_payload(&eth_packet);
                    match capresult {
                        Ok(cap) => match cap {
                            Capture::IP(ip_cap) => {
                                println!("<<< {} >>>", ip_cap);
                                if sender.blocking_send(ip_cap).is_err() {
                                    break;
                                }
                            }
                            _ => {} //TODO: ARP captures. Need separate ARP table
                        },
                        Err(e) => println!("Error: {}", e),
                    }
                }
            }
            Err(e) => {
                panic!("An error occured while reading: {}", e);
            }
        }
    }
}
