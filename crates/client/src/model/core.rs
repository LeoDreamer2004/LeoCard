use super::mahjong::MahjongClientState;
use super::qigui523::QiGuiClientState;
use super::shengji::ShengjiClientState;
use super::texas_holdem::TexasHoldemClientState;
use super::types::{PendingEvents, Sequenced};
use super::uno::UnoClientState;
use leocard_protocol::{
    AvatarId, ClientCommand, ClientMessage, GameEvent, GameRules, GameSnapshot, LobbySnapshot,
    MatchId, PROTOCOL_VERSION, PlayerId, PlayerReferenceChange, RejectReason, RequestId, Revision,
    RoomId, ServerEvent, ServerMessage,
};
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub(super) struct GameClientStates {
    pub(super) qigui523: QiGuiClientState,
    pub(super) texas_holdem: TexasHoldemClientState,
    pub(super) shengji: ShengjiClientState,
    pub(super) uno: UnoClientState,
    pub(super) mahjong: MahjongClientState,
}

#[derive(Clone, Debug)]
pub struct ClientModel {
    pub(super) room_id: RoomId,
    pub(super) host_port: Option<u16>,
    pub(super) next_request: u64,
    pub(super) latest_revision: Revision,
    pub(super) you: Option<PlayerId>,
    pub(super) lobby: Option<LobbySnapshot>,
    pub(super) game: Option<GameSnapshot>,
    pub(super) rules: Option<GameRules>,
    pub(super) avatars: HashMap<AvatarId, Vec<u8>>,
    pub(super) games: GameClientStates,
    pub(super) active_match_id: Option<MatchId>,
    pub(super) room_closed: bool,
    pub(super) left_room: bool,
    pub(super) rejection: Sequenced<RejectReason>,
    pub(super) notice: Sequenced<String>,
    pub(super) pending: PendingEvents,
    pub(super) last_finished_match: Option<(MatchId, Vec<PlayerReferenceChange>)>,
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

    pub(super) fn prepare_game_snapshot(
        &mut self,
        match_id: MatchId,
        host_port: u16,
        you: PlayerId,
    ) -> bool {
        let new_match = self.active_match_id != Some(match_id);
        self.host_port = Some(host_port);
        self.active_match_id = Some(match_id);
        self.you = Some(you);
        self.lobby = None;
        self.rejection.value = None;
        new_match
    }

    pub(super) fn store_game_snapshot(&mut self, snapshot: impl Into<GameSnapshot>) {
        self.game = Some(snapshot.into());
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
                self.pending.player_interactions.push(interaction);
            }
            ServerEvent::ChatMessage(message) => {
                self.pending.chat_messages.push(message);
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
