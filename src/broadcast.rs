use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use itertools::{Either, Itertools};
use pnet::datalink::interfaces;
use pnet::ipnetwork::IpNetwork;

#[derive(Debug)]
pub struct BroadcastPacket {
    protocol_name: &'static str,
    device_id: String,
    device_name: String,
    retransmit: bool,
    port: u16,
    addresses: Vec<IpAddr>
}

// impl BroadcastPacket {
//     pub fn new(device_id: String, device_name: String, retransmit: bool, port: u16, addresses: Vec<IpAddr>) -> Self {
//         // BroadcastPacket {
//         //     protocol_name: PROTOCOL_NAME,
//         //     device_id,
//         //     device_name,
//         //     retransmit,
//         //     port,
//         //     addresses
//         // }
//     }
//
//     pub fn get_ip_addrs() -> (Vec<Ipv4Addr>, Vec<Ipv6Addr>) {
//         interfaces()
//             .into_iter()
//             .filter(|e| e.is_up() && !e.is_loopback() && !e.ips.is_empty())
//             .flat_map(|e| e.ips)
//             .partition_map(|e| match e {
//                 IpNetwork::V4(x) => Either::Left(x.ip()),
//                 IpNetwork::V6(x) => Either::Right(x.ip())
//             })
//     }
// }

// impl Default for BroadcastPacket {
//     fn default() -> Self {
//         BroadcastPacket {
//             protocol_name: PROTOCOL_NAME,
//             device_id,
//             device_name,
//             retransmit,
//             port,
//             addresses
//         }
//     }
// }