use crate::{ClassifiedPlay, GameState, PlayRecord, QiGuiCard, QiGuiPlayerId, QiGuiRuleSet};

/// Own hand and public trick history; opponents' concealed hands are excluded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QiGui523BotObservation {
    pub(super) hand: Vec<QiGuiCard>,
    pub(super) current_play: Option<ClassifiedPlay>,
    pub(super) played_cards: Vec<QiGuiCard>,
    pub(super) rules: QiGuiRuleSet,
}

impl QiGui523BotObservation {
    pub fn from_game(game: &GameState, player: QiGuiPlayerId) -> Option<Self> {
        let trick = game.trick()?;
        if trick.current_player() != player {
            return None;
        }
        Some(Self {
            hand: game.player(player)?.hand().to_vec(),
            current_play: trick.winning_play().cloned(),
            played_cards: trick
                .records()
                .iter()
                .flat_map(|record| match record {
                    PlayRecord::Played { play, .. } => play.cards(),
                    PlayRecord::Passed { .. } => &[],
                })
                .copied()
                .collect(),
            rules: *game.rules(),
        })
    }
}
