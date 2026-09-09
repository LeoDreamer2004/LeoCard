use super::ClientModel;
use leocard_mahjong::MahjongRuleSet;
use leocard_protocol::RoomId;
use leocard_protocol::{
    AvatarId, ChatMessage, GameCommand, GameKind, GameRules, GameSnapshot, LobbySnapshot,
    MahjongSnapshot, MatchId, PlayerGameProfiles, PlayerId, PlayerInteraction,
    PlayerReferenceChange, QiGui523Command, QiGui523Snapshot, RejectReason, Revision,
    ShengjiCommand, ShengjiSnapshot, TexasHoldemCommand, TexasHoldemSnapshot, UnoCommand,
    UnoSnapshot,
};
use leocard_qigui523::QiGuiRuleSet;
use leocard_shengji::ShengjiRuleSet;
use leocard_texas_holdem::TexasHoldemRuleSet;
use leocard_uno::UnoRuleSet;
use std::collections::HashMap;

macro_rules! game_accessors {
    ($(($game:ident, $snapshot:ty, $snapshot_projection:ident, $rules:ident, $rule_set:ty, $rules_projection:ident)),+ $(,)?) => {
        $(
            pub fn $game(&self) -> Option<&$snapshot> {
                self.game.as_ref().and_then(GameSnapshot::$snapshot_projection)
            }

            pub fn $rules(&self) -> Option<&$rule_set> {
                self.rules.as_ref().and_then(GameRules::$rules_projection)
            }
        )+
    };
}

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
    pub fn is_host(&self) -> bool {
        self.you == self.host
    }
}

impl ClientModel {
    pub fn phase(&self) -> ClientPhaseRef<'_> {
        if self.room_closed || self.left_room {
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
        macro_rules! meta {
            ($kind:expr, $game:expr) => {{
                let game = $game;
                ActiveGameMeta {
                    kind: $kind,
                    match_id: game.match_id,
                    you: game.you,
                    host: game.host,
                    players: game.players.iter().map(|player| player.id).collect(),
                }
            }};
        }
        Some(match self.game.as_ref()? {
            GameSnapshot::QiGui523(game) => meta!(GameKind::QiGui523, game),
            GameSnapshot::TexasHoldem(game) => meta!(GameKind::TexasHoldem, game),
            GameSnapshot::Shengji(game) => meta!(GameKind::Shengji, game),
            GameSnapshot::Uno(game) => meta!(GameKind::Uno, game),
            GameSnapshot::Mahjong(game) => meta!(GameKind::Mahjong, game),
        })
    }

    pub fn player_name(&self, id: PlayerId) -> Option<&str> {
        macro_rules! find_name {
            ($players:expr) => {
                $players
                    .iter()
                    .find(|player| player.id == id)
                    .map(|player| player.name.as_str())
            };
        }
        match self.phase() {
            ClientPhaseRef::Lobby(lobby) => find_name!(lobby.players),
            ClientPhaseRef::Playing(GameSnapshot::QiGui523(game)) => find_name!(game.players),
            ClientPhaseRef::Playing(GameSnapshot::TexasHoldem(game)) => find_name!(game.players),
            ClientPhaseRef::Playing(GameSnapshot::Shengji(game)) => find_name!(game.players),
            ClientPhaseRef::Playing(GameSnapshot::Uno(game)) => find_name!(game.players),
            ClientPhaseRef::Playing(GameSnapshot::Mahjong(game)) => find_name!(game.players),
            ClientPhaseRef::Idle | ClientPhaseRef::Closed => None,
        }
    }

    pub fn has_player(&self, id: PlayerId) -> bool {
        self.player_name(id).is_some()
    }

    pub fn local_game_profiles(&self) -> Option<&PlayerGameProfiles> {
        macro_rules! local_profiles {
            ($game:expr) => {{
                let game = $game;
                game.players
                    .iter()
                    .find(|player| player.id == game.you)
                    .map(|player| &player.game_profiles)
            }};
        }
        match self.game.as_ref()? {
            GameSnapshot::QiGui523(game) => local_profiles!(game),
            GameSnapshot::TexasHoldem(game) => local_profiles!(game),
            GameSnapshot::Shengji(game) => local_profiles!(game),
            GameSnapshot::Uno(game) => local_profiles!(game),
            GameSnapshot::Mahjong(game) => local_profiles!(game),
        }
    }

    pub fn game_rules(&self) -> Option<&GameRules> {
        self.rules.as_ref()
    }

    pub fn toggle_auto_play_command(&self) -> Option<GameCommand> {
        macro_rules! toggle {
            ($snapshot:expr, $command:expr) => {{
                let game = $snapshot;
                let enabled = game
                    .players
                    .iter()
                    .find(|player| player.id == game.you)
                    .is_some_and(|player| player.auto_play);
                Some($command(!enabled).into())
            }};
        }

        match self.game.as_ref()? {
            GameSnapshot::QiGui523(game) => {
                toggle!(game, |enabled| QiGui523Command::SetAutoPlay { enabled })
            }
            GameSnapshot::TexasHoldem(game) => {
                toggle!(game, |enabled| TexasHoldemCommand::SetAutoPlay { enabled })
            }
            GameSnapshot::Shengji(game) => {
                toggle!(game, |enabled| ShengjiCommand::SetAutoPlay { enabled })
            }
            GameSnapshot::Uno(game) => {
                toggle!(game, |enabled| UnoCommand::SetAutoPlay { enabled })
            }
            GameSnapshot::Mahjong(_) => None,
        }
    }

    game_accessors!(
        (
            qigui523_game,
            QiGui523Snapshot,
            qigui523,
            qigui523_rules,
            QiGuiRuleSet,
            qigui523
        ),
        (
            texas_holdem_game,
            TexasHoldemSnapshot,
            texas_holdem,
            texas_holdem_rules,
            TexasHoldemRuleSet,
            texas_holdem
        ),
        (
            shengji_game,
            ShengjiSnapshot,
            shengji,
            shengji_rules,
            ShengjiRuleSet,
            shengji
        ),
        (uno_game, UnoSnapshot, uno, uno_rules, UnoRuleSet, uno),
        (
            mahjong_game,
            MahjongSnapshot,
            mahjong,
            mahjong_rules,
            MahjongRuleSet,
            mahjong
        ),
    );

    pub fn avatars(&self) -> &HashMap<AvatarId, Vec<u8>> {
        &self.avatars
    }

    pub fn room_closed(&self) -> bool {
        self.room_closed
    }

    pub fn left_room(&self) -> bool {
        self.left_room
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
