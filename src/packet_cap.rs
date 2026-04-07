use chrono::{DateTime, Local};
use derivative::Derivative;
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, NetworkInterface, interfaces};
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
use std::time::Duration;
use thiserror::Error;

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

#[derive(Derivative)]
#[derivative(Debug)]
pub struct IpCapture {
    timestamp: DateTime<Local>,
    source: IpAddr,
    ethernet_frame_type: EtherType,
    transport_protocol: IpNextHeaderProtocol,
    length: u16,
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
    timestamp: DateTime<Local>,
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
enum Capture {
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
struct DisplayIpAddr<'a>(&'a Vec<IpNetwork>);

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
struct DisplayMacAddr(Option<MacAddr>);

impl fmt::Display for DisplayMacAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Some(mac) => write!(f, "{}", mac),
            None => write!(f, "N/A"),
        }
    }
}

pub fn cmd_list() {
    let active_interfaces: Vec<datalink::NetworkInterface> = datalink::interfaces()
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
    let ivec = interfaces();
    let iface = ivec.into_iter().find(|iface| iface.name == input);
    iface
}

pub fn cmd_info(iface_name: &str) {
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
                timestamp: Local::now(),
                source: IpAddr::from(ipv4.get_source()),
                length: payload_len,
                ethernet_frame_type: EtherTypes::Ipv4,
                transport_protocol: ipv4.get_next_level_protocol(),
                payload: ipv4.payload().to_vec(),
            }))
        }
        EtherTypes::Ipv6 => {
            let ipv6 = Ipv6Packet::new(&eth_pkt.payload()).ok_or(CaptureError::MalformedIpv6)?;
            Ok(Capture::IP(IpCapture {
                timestamp: Local::now(),
                source: IpAddr::from(ipv6.get_source()),
                length: ipv6.get_payload_length(),
                ethernet_frame_type: EtherTypes::Ipv6,
                transport_protocol: ipv6.get_next_header(),
                payload: ipv6.payload().to_vec(),
            }))
        }
        EtherTypes::Arp => {
            let arp = ArpPacket::new(&eth_pkt.payload()).ok_or(CaptureError::MalformedArp)?;
            Ok(Capture::ARP(ArpCapture {
                timestamp: Local::now(),
                source: IpAddr::from(arp.get_sender_proto_addr()),
                source_mac: MacAddr::from(arp.get_sender_hw_addr()),
                operation: arp.get_operation(),
                hardware_type: arp.get_hardware_type(),
            }))
        }
        other => Err(CaptureError::UnsupportedProtocol(other)), //TODO: Consider other Ethertypes
    }
}

//Simply bind and print info to stdout. No DB entries. Use for testing
//interfaces and commands, or for just watching the data flow
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
                    let capresult = parse_payload(&eth_packet);
                    match capresult {
                        Ok(cap) => println!("<<< {} >>>", cap),
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

struct KnownInterfaces {
    unique_iface_count: u16,
    unique_ifaces: Vec<NetworkInterface>,
}

struct IpRecord {
    mac: MacAddr,
    ipv4: Option<Ipv4Addr>,
    ipv6: Option<Ipv6Addr>,
}

struct IpRecordList {
    unique_addr_count: u16,
    unique_addrs: Vec<IpRecord>,
}

struct CapLog<'a> {
    cap_count: u16,
    cap_duration: std::time::Duration,
    ip_addrs: &'a IpRecordList,
}

pub fn bind_and_catalog(i: &NetworkInterface, ip_list: &mut IpRecordList) {
    //TODO: Forget iplist. We'll pass a DB handle with that + more once it's ready
    let (mut _tx, mut rx) = match datalink::channel(&i, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled Channel Type"),
        Err(e) => panic!("Error binding to interface: {}", e),
    };
    let mut log = CapLog {
        cap_count: 0,
        cap_duration: Duration::from_secs(0),
        ip_addrs: &ip_list,
    };
    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(eth_packet) = EthernetPacket::new(&packet) {
                    let capresult = parse_payload(&eth_packet);
                    match capresult {
                        Ok(cap) => {
                            log.cap_count += 1;
                            match cap {
                                Capture::IP(ipcap) => {}
                                _ => {} //TODO: Cross-reference unique Ip addresses known by local db.
                            }
                            println!("COUNT: {}", log.cap_count);
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                }
            }
            Err(e) => {
                panic!("EEEEEEEE{}", e);
            }
        }
    }
}
