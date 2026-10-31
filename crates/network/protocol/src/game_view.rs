use crate::players::PlayerRoster;
use crate::{
    BlackjackCommand, DoudizhuCommand, GameCommand, GameRules, GameSnapshot, MahjongCommand,
    MahjongPhaseView, MatchId, PlayerPublicInfo, PlayerReferenceChange, QiGui523Command,
    QiGui523PhaseView, ShengjiCommand, ShengjiPhaseView, TexasCommand, TexasPhaseView, UnoCommand,
    UnoPhaseView,
};
use leocard_game_common::PlayerId;

macro_rules! snapshot_field {
    ($self:expr, $game:ident, $body:expr) => {
        match $self {
            GameSnapshot::QiGui523($game) => $body,
            GameSnapshot::Texas($game) => $body,
            GameSnapshot::Shengji($game) => $body,
            GameSnapshot::Uno($game) => $body,
            GameSnapshot::Mahjong($game) => $body,
            GameSnapshot::McGuess($game) => $body,
            GameSnapshot::Blackjack($game) => $body,
            GameSnapshot::Doudizhu($game) => $body,
        }
    };
}

impl GameSnapshot {
    fn player_roster(&self) -> &dyn PlayerRoster {
        match self {
            Self::QiGui523(snapshot) => &snapshot.players,
            Self::Doudizhu(snapshot) => &snapshot.players,
            Self::Texas(snapshot) => &snapshot.players,
            Self::Shengji(snapshot) => &snapshot.players,
            Self::Uno(snapshot) => &snapshot.players,
            Self::Mahjong(snapshot) => &snapshot.players,
            Self::McGuess(snapshot) => &snapshot.players,
            Self::Blackjack(snapshot) => &snapshot.players,
        }
    }

    pub fn players(&self) -> impl ExactSizeIterator<Item = &PlayerPublicInfo> {
        let roster = self.player_roster();
        (0..roster.player_count()).map(move |index| roster.player_info(index))
    }

    pub fn player(&self, id: PlayerId) -> Option<&PlayerPublicInfo> {
        self.players().find(|player| player.id == id)
    }

    pub const fn host(&self) -> PlayerId {
        match self {
            Self::QiGui523(snapshot) => snapshot.host,
            Self::Doudizhu(snapshot) => snapshot.host,
            Self::Texas(snapshot) => snapshot.host,
            Self::Shengji(snapshot) => snapshot.host,
            Self::Uno(snapshot) => snapshot.host,
            Self::Mahjong(snapshot) => snapshot.host,
            Self::McGuess(snapshot) => snapshot.host,
            Self::Blackjack(snapshot) => snapshot.host,
        }
    }

    pub fn toggle_auto_play_command(&self) -> Option<GameCommand> {
        macro_rules! toggle {
            ($snapshot:expr, $command:ident) => {{
                let game = $snapshot;
                let player = game
                    .players
                    .iter()
                    .find(|player| player.player.id == self.you())?;
                Some(
                    $command::SetAutoPlay {
                        enabled: !player.auto_play,
                    }
                    .into(),
                )
            }};
        }
        match self {
            Self::QiGui523(snapshot) => toggle!(snapshot, QiGui523Command),
            Self::Doudizhu(snapshot) => toggle!(snapshot, DoudizhuCommand),
            Self::Texas(snapshot) => toggle!(snapshot, TexasCommand),
            Self::Shengji(snapshot) => toggle!(snapshot, ShengjiCommand),
            Self::Uno(snapshot) => toggle!(snapshot, UnoCommand),
            Self::Mahjong(snapshot) => toggle!(snapshot, MahjongCommand),
            Self::Blackjack(snapshot) => toggle!(snapshot, BlackjackCommand),
            Self::McGuess(_) => None,
        }
    }

    pub fn host_port(&self) -> u16 {
        snapshot_field!(self, game, game.host_port)
    }

    pub fn rules(&self) -> Option<GameRules> {
        Some(match self {
            Self::QiGui523(_) | Self::Texas(_) => return None,
            Self::Shengji(game) => GameRules::Shengji(game.rules),
            Self::Uno(game) => GameRules::Uno(game.rules),
            Self::Mahjong(game) => GameRules::Mahjong(game.rules),
            Self::McGuess(game) => GameRules::McGuess(game.rules),
            Self::Blackjack(game) => GameRules::Blackjack(game.rules),
            Self::Doudizhu(game) => GameRules::Doudizhu(game.rules),
        })
    }

    pub fn settlement(&self) -> Option<(MatchId, &[PlayerReferenceChange])> {
        let (id, changes) = match self {
            Self::QiGui523(game) => match &game.phase {
                QiGui523PhaseView::Finished {
                    match_id,
                    reference_changes,
                    ..
                } => (*match_id, reference_changes),
                _ => return None,
            },
            Self::Texas(game) => match &game.phase {
                TexasPhaseView::HandComplete {
                    tournament_complete: true,
                    reference_changes,
                    ..
                } => (game.match_id, reference_changes),
                _ => return None,
            },
            Self::Shengji(game) => match &game.phase {
                ShengjiPhaseView::Finished { result, .. } => {
                    (result.settlement_id, &result.reference_changes)
                }
                _ => return None,
            },
            Self::Uno(game) => match &game.phase {
                UnoPhaseView::Finished {
                    reference_changes, ..
                } => (game.match_id, reference_changes),
                _ => return None,
            },
            Self::Mahjong(game) => match &game.phase {
                MahjongPhaseView::Finished { result } if result.match_complete => {
                    (game.match_id, &result.reference_changes)
                }
                _ => return None,
            },
            Self::McGuess(game) if self.match_complete() => {
                (game.match_id, &game.reference_changes)
            }
            Self::Blackjack(game) if self.match_complete() => {
                (game.match_id, &game.reference_changes)
            }
            Self::Doudizhu(game) if self.match_complete() => {
                (game.match_id, &game.reference_changes)
            }
            _ => return None,
        };
        Some((id, changes))
    }
}
