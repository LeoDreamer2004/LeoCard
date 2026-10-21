use super::packet::{AnnouncementPacket, DISCOVERY_PORT};
use leocard_protocol::{RoomId, RoomInfo};
use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tokio::net::UdpSocket;

pub(crate) struct LanAnnouncer {
    socket: UdpSocket,
    server_id: u64,
    port: u16,
    broadcast: bool,
}

impl LanAnnouncer {
    pub(crate) async fn new(address: SocketAddr) -> io::Result<Self> {
        if !address.is_ipv4() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "LAN discovery requires IPv4",
            ));
        }
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
        socket.set_broadcast(true)?;
        Ok(Self {
            socket,
            server_id: fastrand::u64(..),
            port: address.port(),
            broadcast: !address.ip().is_loopback(),
        })
    }

    pub(crate) async fn announce(&self, room_id: RoomId, info: RoomInfo) {
        let Some(bytes) = AnnouncementPacket::encode(self.server_id, room_id, self.port, info)
        else {
            return;
        };
        // Loopback also discovers other windows on a machine without an active LAN interface.
        for ip in [Ipv4Addr::BROADCAST, Ipv4Addr::LOCALHOST] {
            if ip == Ipv4Addr::BROADCAST && !self.broadcast {
                continue;
            }
            let _ = self
                .socket
                .send_to(&bytes, SocketAddrV4::new(ip, DISCOVERY_PORT))
                .await;
        }
    }
}
