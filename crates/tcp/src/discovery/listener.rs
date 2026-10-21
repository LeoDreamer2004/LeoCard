use super::packet::{AnnouncementPacket, DISCOVERY_PORT, MAX_PACKET};
use crate::{TcpClient, TcpError};
use leocard_protocol::{
    ClientCommand, ClientMessage, MAX_AVATAR_BYTES, MAX_PLAYER_NAME_CHARS, PROTOCOL_VERSION,
    RequestId, RoomId, RoomInfo, ServerEvent,
};
use socket2::{Domain, Protocol, Socket, Type};
use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;

#[derive(Clone, Debug)]
pub struct LanRoomAnnouncement {
    pub server_id: u64,
    pub address: SocketAddr,
    pub room_id: RoomId,
    pub info: RoomInfo,
}

pub struct LanDiscoverySocket(UdpSocket);

impl LanDiscoverySocket {
    pub fn bind() -> io::Result<Self> {
        Self::bind_port(DISCOVERY_PORT)
    }

    pub(super) fn bind_port(port: u16) -> io::Result<Self> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        // Several clients on one computer must all be able to receive broadcast packets.
        socket.set_reuse_address(true)?;
        socket.set_nonblocking(true)?;
        socket.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port).into())?;
        Ok(Self(UdpSocket::from_std(socket.into())?))
    }

    pub async fn receive(&self) -> io::Result<LanRoomAnnouncement> {
        let mut bytes = [0; MAX_PACKET + 1];
        loop {
            let (length, source) = self.0.recv_from(&mut bytes).await?;
            let Some(packet) = AnnouncementPacket::decode(&bytes[..length]) else {
                continue;
            };
            return Ok(LanRoomAnnouncement {
                server_id: packet.server_id,
                // Never trust an advertised IP address to redirect the TCP metadata request.
                address: SocketAddr::new(source.ip(), packet.port),
                room_id: packet.room_id,
                info: packet.info,
            });
        }
    }
}

pub async fn query_room_info(address: SocketAddr, room_id: RoomId) -> Result<RoomInfo, TcpError> {
    timeout(Duration::from_secs(2), async {
        let mut client = TcpClient::connect(address).await?;
        client
            .send(&ClientMessage::new(
                room_id,
                RequestId(0),
                ClientCommand::RequestRoomInfo,
            ))
            .await?;
        let message = client.receive().await?;
        if message.protocol_version == PROTOCOL_VERSION
            && message.room_id == room_id
            && let ServerEvent::RoomInfo(Some(info)) = message.event
            && !info.host_name.trim().is_empty()
            && info.host_name.chars().count() <= MAX_PLAYER_NAME_CHARS
            && info
                .avatar_png
                .as_ref()
                .is_none_or(|png| png.len() <= MAX_AVATAR_BYTES)
        {
            return Ok(info);
        }
        Err(TcpError::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid room discovery response",
        )))
    })
    .await
    .map_err(|_| TcpError::Timeout("room discovery"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use leocard_protocol::GameKind;

    #[tokio::test]
    async fn receive_ignores_unrelated_packets_and_uses_the_advertised_tcp_port() {
        let listener = LanDiscoverySocket::bind_port(0).unwrap();
        let port = listener.0.local_addr().unwrap().port();
        let sender = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
        let target = (Ipv4Addr::LOCALHOST, port);
        sender.send_to(b"unrelated", target).await.unwrap();
        let info = RoomInfo {
            host_name: "房主".to_owned(),
            host_avatar: None,
            avatar_png: None,
            game: GameKind::Mahjong,
            playing: false,
        };
        let bytes = AnnouncementPacket::encode(17, RoomId(9), 54321, info.clone()).unwrap();
        sender.send_to(&bytes, target).await.unwrap();
        let notice = timeout(Duration::from_secs(1), listener.receive())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(notice.server_id, 17);
        assert_eq!(
            notice.address,
            SocketAddr::from((Ipv4Addr::LOCALHOST, 54321))
        );
        assert_eq!(notice.info, info);
    }
}
