use std::collections::HashMap;

use leocard_protocol::{
    AvatarId, ClientCommand, ClientMessage, GameEvent, GameRules, GameSnapshot, LobbySnapshot,
    MatchId, PROTOCOL_VERSION, PlayerId, PlayerReferenceChange, RejectReason, RequestId, Revision,
    RoomId, ServerEvent, ServerMessage,
};

#[cfg(test)]
mod tests;

mod access;
mod mahjong;
mod qigui523;
mod shengji;
mod texas_holdem;
mod types;
mod uno;

use mahjong::MahjongClientState;
use qigui523::QiGuiClientState;
pub use qigui523::ScoreCaptureEffect;
use shengji::ShengjiClientState;
pub use shengji::ShengjiScoreCaptureEffect;
use texas_holdem::TexasHoldemClientState;
use types::{PendingEvents, Sequenced};
use uno::UnoClientState;

#[derive(Clone, Debug, Default)]
struct GameClientStates {
    qigui523: QiGuiClientState,
    texas_holdem: TexasHoldemClientState,
    shengji: ShengjiClientState,
    uno: UnoClientState,
    mahjong: MahjongClientState,
}

#[derive(Clone, Debug)]
pub struct ClientModel {
    room_id: RoomId,
    host_port: Option<u16>,
    next_request: u64,
    latest_revision: Revision,
    you: Option<PlayerId>,
    lobby: Option<LobbySnapshot>,
    game: Option<GameSnapshot>,
    rules: Option<GameRules>,
    avatars: HashMap<AvatarId, Vec<u8>>,
    games: GameClientStates,
    active_match_id: Option<MatchId>,
    room_closed: bool,
    left_room: bool,
    rejection: Sequenced<RejectReason>,
    notice: Sequenced<String>,
    pending: PendingEvents,
    last_finished_match: Option<(MatchId, Vec<PlayerReferenceChange>)>,
}

impl ClientModel {
    pub fn new(room_id: RoomId) -> Self {
        Self {
            room_id,
            host_port: None,
            next_request: 1,
            latest_revision: Revision(0),
            you: None,
            lobby: None,
            game: None,
            rules: None,
            avatars: HashMap::new(),
            games: GameClientStates::default(),
            active_match_id: None,
            room_closed: false,
            left_room: false,
            rejection: Sequenced::default(),
            notice: Sequenced::default(),
            pending: PendingEvents::default(),
            last_finished_match: None,
        }
    }

    pub fn command(&mut self, command: ClientCommand) -> ClientMessage {
        let request_id = RequestId(self.next_request);
        self.next_request += 1;
        ClientMessage::new(self.room_id, request_id, command)
    }

    /// 返回消息是否被接受。错误房间、错误版本和旧修订号均被忽略。
    pub fn apply(&mut self, message: ServerMessage) -> bool {
        if message.protocol_version != PROTOCOL_VERSION
            || message.room_id != self.room_id
            || message.revision < self.latest_revision
        {
            return false;
        }
        self.latest_revision = message.revision;
        match message.event {
            ServerEvent::Joined { you } => {
                self.you = Some(you);
                self.rejection.value = None;
            }
            ServerEvent::AvatarData { id, png } => {
                self.avatars.entry(id).or_insert(png);
            }
            ServerEvent::LobbySnapshot(snapshot) => {
                self.host_port = Some(snapshot.host_port);
                self.room_closed = false;
                self.games = GameClientStates::default();
                self.active_match_id = None;
                self.pending.clear();
                self.rules = Some(snapshot.rules.clone());
                self.lobby = Some(snapshot);
                self.game = None;
                self.rejection.value = None;
            }
            ServerEvent::GameSnapshot(GameSnapshot::QiGui523(snapshot)) => {
                self.apply_qigui523_snapshot(snapshot);
            }
            ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(snapshot)) => {
                self.apply_texas_holdem_snapshot(snapshot);
            }
            ServerEvent::GameSnapshot(GameSnapshot::Shengji(snapshot)) => {
                self.apply_shengji_snapshot(snapshot);
            }
            ServerEvent::GameSnapshot(GameSnapshot::Uno(snapshot)) => {
                self.apply_uno_snapshot(snapshot);
            }
            ServerEvent::GameSnapshot(GameSnapshot::Mahjong(snapshot)) => {
                self.apply_mahjong_snapshot(snapshot);
            }
            ServerEvent::GameEvent(GameEvent::QiGui523(event)) => {
                self.apply_qigui523_event(event);
            }
            ServerEvent::GameEvent(GameEvent::TexasHoldem(event)) => {
                self.apply_texas_holdem_event(event);
            }
            ServerEvent::GameEvent(GameEvent::Shengji(event)) => {
                self.apply_shengji_event(event);
            }
            ServerEvent::GameEvent(GameEvent::Uno(event)) => {
                self.apply_uno_event(event);
            }
            ServerEvent::GameEvent(GameEvent::Mahjong(event)) => {
                self.apply_mahjong_event(event);
            }
            ServerEvent::PlayerInteraction(interaction) => {
                self.pending.player_interactions.push_back(interaction);
            }
            ServerEvent::ChatMessage(message) => {
                self.pending.chat_messages.push_back(message);
            }
            ServerEvent::PlayerLeft { name } => {
                self.notice.publish(format!("{name}退出了游戏"));
            }
            ServerEvent::LeftRoom => {
                self.left_room = true;
            }
            ServerEvent::RoomClosed => {
                self.room_closed = true;
            }
            ServerEvent::Rejected { reason } => {
                self.rejection.publish(reason);
            }
            ServerEvent::Heartbeat => {}
        }
        true
    }
}
