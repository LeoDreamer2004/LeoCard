use super::event::{EventSequence, PendingEvents};
use leocard_game_common::PlayerId;
use leocard_protocol::{
    AvatarId, ClientCommand, ClientMessage, GameRules, GameSnapshot, LobbySnapshot, MatchId,
    PROTOCOL_VERSION, PlayerReferenceChange, RejectReason, RequestId, Revision, RoomId,
    ServerEvent, ServerMessage,
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RoomExitReason {
    Left,
    Closed,
    Kicked,
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
    pub(super) room_exit: Option<RoomExitReason>,
    pub(super) rejection: EventSequence<RejectReason>,
    pub(super) notice: EventSequence<String>,
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
            room_exit: None,
            rejection: EventSequence::default(),
            notice: EventSequence::default(),
            pending: PendingEvents::default(),
            last_finished_match: None,
        }
    }

    pub fn command(&mut self, command: ClientCommand) -> ClientMessage {
        let request_id = RequestId(self.next_request);
        self.next_request += 1;
        ClientMessage::new(self.room_id, request_id, command)
    }

    fn apply_game_snapshot(&mut self, snapshot: GameSnapshot) {
        if self.game.as_ref().map(GameSnapshot::match_id) != Some(snapshot.match_id()) {
            self.pending.player_interactions.clear();
        }
        self.host_port = Some(snapshot.host_port());
        // A spectator's Joined identity differs from the private view they are watching.
        if self.lobby.is_some() && !self.is_spectating() {
            self.you = Some(snapshot.you());
        } else {
            self.you.get_or_insert(snapshot.you());
        }
        self.lobby = None;
        self.rejection.value = None;
        if let Some(rules) = snapshot.rules() {
            self.rules = Some(rules);
        }
        if let Some((id, changes)) = snapshot.settlement() {
            self.last_finished_match = Some((id, changes.to_vec()));
        }
        self.game = Some(snapshot);
    }

    /// 返回消息是否被接受。错误房间、错误版本和旧修订号均被忽略。
    pub fn apply(&mut self, message: ServerMessage) -> bool {
        self.apply_with_events(message, |_, _| {})
    }

    /// Observe accepted live events before applying state changes; rejected messages
    /// never reach observers. Feature-specific consumers own their own state.
    pub fn apply_with_events(
        &mut self,
        message: ServerMessage,
        mut observe: impl FnMut(Option<PlayerId>, &ServerMessage),
    ) -> bool {
        if message.protocol_version != PROTOCOL_VERSION
            || message.room_id != self.room_id
            || message.revision < self.latest_revision
        {
            return false;
        }
        self.latest_revision = message.revision;
        observe(self.you, &message);
        match message.event {
            ServerEvent::RulesSnapshot(rules) => self.rules = Some(rules),
            ServerEvent::AchievementUnlocked(_)
            | ServerEvent::GameStarted { .. }
            | ServerEvent::RoomInfo(_)
            | ServerEvent::Notice { .. } => {}
            ServerEvent::Joined { you } => {
                self.you = Some(you);
                self.rejection.value = None;
            }
            ServerEvent::AvatarData { id, png } => {
                self.avatars.entry(id).or_insert(png);
            }
            ServerEvent::LobbySnapshot(snapshot) => {
                self.host_port = Some(snapshot.host_port);
                self.room_exit = None;
                if self.lobby.is_none() {
                    self.pending.clear();
                }
                self.rules = Some(snapshot.rules.clone());
                self.lobby = Some(snapshot);
                self.game = None;
                self.rejection.value = None;
            }
            ServerEvent::GameSnapshot(snapshot) => self.apply_game_snapshot(snapshot),
            ServerEvent::GameEvent(_) => {}
            ServerEvent::PlayerInteraction(interaction) => {
                self.pending.player_interactions.push(interaction);
            }
            ServerEvent::ChatMessage(message) => {
                self.pending.chat_messages.push(message);
            }
            ServerEvent::PlayerLeft { name } => {
                self.notice.publish(format!("{name}退出了游戏"));
            }
            ServerEvent::PlayerKicked { name } => {
                self.notice.publish(format!("{name}已被房主踢出"));
            }
            ServerEvent::SpectatorJoined { name } => {
                self.notice.publish(format!("{name} 正在旁观游戏"));
            }
            ServerEvent::LeftRoom => {
                self.room_exit = Some(RoomExitReason::Left);
            }
            ServerEvent::RoomClosed => {
                self.room_exit = Some(RoomExitReason::Closed);
            }
            ServerEvent::KickedFromRoom => {
                self.room_exit = Some(RoomExitReason::Kicked);
            }
            ServerEvent::Rejected { reason } => {
                self.rejection.publish(reason);
            }
            ServerEvent::Heartbeat => {}
        }
        true
    }
}
