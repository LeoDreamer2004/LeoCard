use super::state::NETWORK_ROOM_ID;
use leocard_protocol::{GameKind, PROTOCOL_VERSION, ServerEvent, ServerMessage};

#[derive(Default)]
pub(super) struct Handshake {
    joined: bool,
    rules: Option<GameKind>,
}

impl Handshake {
    pub(super) fn accept(&mut self, message: &ServerMessage) -> Result<bool, String> {
        validate_message(message)?;
        Ok(match &message.event {
            ServerEvent::Joined { .. } => {
                self.joined = true;
                false
            }
            ServerEvent::RulesSnapshot(snapshot) => {
                self.rules = Some(snapshot.kind());
                false
            }
            ServerEvent::Rejected { reason } => return Err(reason.to_string()),
            ServerEvent::RoomClosed | ServerEvent::LeftRoom => return Err("房间已关闭".to_owned()),
            ServerEvent::KickedFromRoom => return Err("你已被房主踢出".to_owned()),
            ServerEvent::LobbySnapshot(snapshot) => self.joined && snapshot.host.is_some(),
            ServerEvent::GameSnapshot(snapshot) => {
                self.joined && self.rules == Some(snapshot.kind())
            }
            _ => false,
        })
    }
}

pub(super) fn validate_message(message: &ServerMessage) -> Result<(), String> {
    if message.protocol_version != PROTOCOL_VERSION {
        return Err("与房主的协议版本不一致，请使用相同版本的游戏".to_owned());
    }
    if message.room_id != NETWORK_ROOM_ID {
        return Err("房主返回了错误的房间编号".to_owned());
    }
    Ok(())
}
