use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, NetworkInterface, interfaces};
use pnet::packet::ethernet::{EthernetPacket, MutableEthernetPacket};
use pnet::packet::{self, ip};
use pnet::packet::{MutablePacket, Packet};

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

pub fn bind_and_listen(i: &NetworkInterface) {
    let (mut _tx, mut rx) = match datalink::channel(&i, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled Channel Type"),
        Err(e) => panic!("Error binding to interface: {}", e),
    };
    loop {
        match rx.next() {
            Ok(packet) => {
                println!("Recieved Packet: {} Bytes", packet.len());
            }
            Err(e) => {
                panic!("An error occured while reading: {}", e);
            }
        }
    }
}
