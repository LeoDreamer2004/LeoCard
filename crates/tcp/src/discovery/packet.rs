use leocard_protocol::{
    MAX_PLAYER_NAME_CHARS, PROTOCOL_VERSION, RoomId, RoomInfo, decode_frame, encode_frame,
};
use serde::{Deserialize, Serialize};

pub(super) const DISCOVERY_PORT: u16 = 52301;
pub(super) const MAX_PACKET: usize = 1200;
const MAGIC: [u8; 8] = *b"LEOCARD1";

#[derive(Deserialize, Serialize)]
pub(super) struct AnnouncementPacket {
    magic: [u8; 8],
    version: u16,
    pub server_id: u64,
    pub room_id: RoomId,
    pub port: u16,
    pub info: RoomInfo,
}

impl AnnouncementPacket {
    pub(super) fn encode(
        server_id: u64,
        room_id: RoomId,
        port: u16,
        mut info: RoomInfo,
    ) -> Option<Vec<u8>> {
        // Keep broadcasts below one datagram; fetch an avatar once over framed TCP.
        info.avatar_png = None;
        let packet = Self {
            magic: MAGIC,
            version: PROTOCOL_VERSION,
            server_id,
            room_id,
            port,
            info,
        };
        encode_frame(&packet)
            .ok()
            .filter(|bytes| bytes.len() <= MAX_PACKET)
    }

    pub(super) fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() > MAX_PACKET {
            return None;
        }
        let packet: Self = decode_frame(bytes).ok()?;
        (packet.magic == MAGIC
            && packet.version == PROTOCOL_VERSION
            && packet.port != 0
            && !packet.info.host_name.trim().is_empty()
            && packet.info.host_name.chars().count() <= MAX_PLAYER_NAME_CHARS
            && packet.info.avatar_png.is_none())
        .then_some(packet)
    }
}
