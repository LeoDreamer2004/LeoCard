use super::ClientModel;
use super::core::RoomExitReason;
use leocard_game_common::PlayerId;
use leocard_protocol::{
    AvatarId, ChatMessage, GameCommand, GameKind, GameRules, GameSnapshot, LobbySnapshot, MatchId,
    PlayerGameProfiles, PlayerInteraction, PlayerPublicInfo, PlayerReferenceChange, RejectReason,
    Revision, RoomId,
};
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveGameMeta {
    pub kind: GameKind,
    pub match_id: MatchId,
    pub you: PlayerId,
    pub host: PlayerId,
    pub players: Vec<PlayerId>,
}

#[derive(Clone, Copy, Debug)]
pub enum ClientPhaseRef<'a> {
    Idle,
    Lobby(&'a LobbySnapshot),
    Playing(&'a GameSnapshot),
    Closed,
}

impl ActiveGameMeta {
    pub fn is_spectating(&self) -> bool {
        !self.players.contains(&self.you)
    }

    pub fn is_host(&self) -> bool {
        self.you == self.host
    }
}

impl ClientModel {
    pub fn is_spectating(&self) -> bool {
        self.active_game_meta()
            .is_some_and(|game| game.is_spectating())
            || self
                .lobby
                .as_ref()
                .zip(self.you)
                .is_some_and(|(lobby, you)| {
                    !lobby.players.iter().any(|player| player.player.id == you)
                })
    }

    pub fn phase(&self) -> ClientPhaseRef<'_> {
        if self.room_exit.is_some() {
            ClientPhaseRef::Closed
        } else if let Some(lobby) = self.lobby.as_ref() {
            ClientPhaseRef::Lobby(lobby)
        } else if let Some(game) = self.game.as_ref() {
            ClientPhaseRef::Playing(game)
        } else {
            ClientPhaseRef::Idle
        }
    }

    pub fn room_id(&self) -> RoomId {
        self.room_id
    }

    pub fn host_port(&self) -> Option<u16> {
        self.host_port
    }

    pub fn latest_revision(&self) -> Revision {
        self.latest_revision
    }

    pub fn you(&self) -> Option<PlayerId> {
        self.you
    }

    pub fn lobby(&self) -> Option<&LobbySnapshot> {
        self.lobby.as_ref()
    }

    pub fn game_snapshot(&self) -> Option<&GameSnapshot> {
        self.game.as_ref()
    }

    pub fn active_game_meta(&self) -> Option<ActiveGameMeta> {
        let game = self.game.as_ref()?;
        Some(ActiveGameMeta {
            kind: game.kind(),
            match_id: game.match_id(),
            you: self.you.unwrap_or(game.you()),
            host: game.host(),
            players: game.players().map(|player| player.id).collect(),
        })
    }

    pub fn player(&self, id: PlayerId) -> Option<&PlayerPublicInfo> {
        match self.phase() {
            ClientPhaseRef::Lobby(lobby) => lobby
                .players
                .iter()
                .map(|view| &view.player)
                .find(|player| player.id == id),
            ClientPhaseRef::Playing(game) => game.player(id),
            ClientPhaseRef::Idle | ClientPhaseRef::Closed => None,
        }
    }

    pub fn player_name(&self, id: PlayerId) -> Option<&str> {
        Some(&self.player(id)?.name)
    }

    pub fn has_player(&self, id: PlayerId) -> bool {
        self.player_name(id).is_some()
    }

    pub fn player_avatar(&self, id: PlayerId) -> Option<AvatarId> {
        self.player(id)?.avatar
    }

    pub fn player_game_profiles(&self, id: PlayerId) -> Option<&PlayerGameProfiles> {
        Some(&self.player(id)?.game_profiles)
    }

    pub fn local_game_profiles(&self) -> Option<&PlayerGameProfiles> {
        self.player_game_profiles(self.you()?)
    }

    pub fn game_rules(&self) -> Option<&GameRules> {
        self.rules.as_ref()
    }

    pub fn toggle_auto_play_command(&self) -> Option<GameCommand> {
        self.game.as_ref()?.toggle_auto_play_command()
    }

    pub fn avatars(&self) -> &HashMap<AvatarId, Vec<u8>> {
        &self.avatars
    }

    pub fn room_closed(&self) -> bool {
        self.room_exit == Some(RoomExitReason::Closed)
    }

    pub fn left_room(&self) -> bool {
        matches!(
            self.room_exit,
            Some(RoomExitReason::Left | RoomExitReason::Kicked)
        )
    }

    pub fn kicked_from_room(&self) -> bool {
        self.room_exit == Some(RoomExitReason::Kicked)
    }

    pub fn last_notice(&self) -> Option<&str> {
        self.notice.value.as_deref()
    }

    pub fn notice_serial(&self) -> u64 {
        self.notice.serial
    }

    pub fn take_player_interactions(&mut self) -> Vec<PlayerInteraction> {
        self.pending.player_interactions.take()
    }

    pub fn take_chat_messages(&mut self) -> Vec<ChatMessage> {
        self.pending.chat_messages.take()
    }

    pub fn last_finished_match(&self) -> Option<(MatchId, &[PlayerReferenceChange])> {
        self.last_finished_match
            .as_ref()
            .map(|(match_id, changes)| (*match_id, changes.as_slice()))
    }

    pub fn last_rejection(&self) -> Option<&RejectReason> {
        self.rejection.value.as_ref()
    }

    /// 每收到一次拒绝消息便递增，即使相邻两次拒绝的内容完全相同。
    pub fn rejection_serial(&self) -> u64 {
        self.rejection.serial
    }
}
