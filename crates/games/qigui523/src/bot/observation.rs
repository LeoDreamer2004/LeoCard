use crate::{QiGuiCard, QiGuiClassifiedPlay, QiGuiGameState, QiGuiPlayRecord, QiGuiRuleSet};
use leocard_game_common::PlayerId;

/// Own hand and public trick history; opponents' concealed hands are excluded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QiGui523BotObservation {
    pub(super) hand: Vec<QiGuiCard>,
    pub(super) current_play: Option<QiGuiClassifiedPlay>,
    pub(super) played_cards: Vec<QiGuiCard>,
    pub(super) rules: QiGuiRuleSet,
}

impl QiGui523BotObservation {
    pub fn from_game(game: &QiGuiGameState, player: PlayerId) -> Option<Self> {
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
                    QiGuiPlayRecord::Played { play, .. } => play.cards(),
                    QiGuiPlayRecord::Passed { .. } => &[],
                })
                .copied()
                .collect(),
            rules: *game.rules(),
        })
    }
}
