use super::QiGuiActionStatistics;
use crate::{QiGuiCard, QiGuiPlayerId, QiGuiRank, QiGuiRuleSet};

impl QiGuiActionStatistics {
    pub(super) fn new(player: QiGuiPlayerId, rules: QiGuiRuleSet) -> Self {
        Self {
            player,
            rules,
            progress: Default::default(),
            played_kind: None,
            played_count: 0,
            diamond_four: false,
            spade_seven_follow: false,
            heaven_over_heaven: false,
            late_bomb: false,
            straight_from_four: false,
            full_hand_play: false,
            bomb_revenge: false,
            captured_fives: 0,
            captured_tens_and_kings: 0,
            captured_points: 0,
            completed_game: false,
            won: false,
            all_points: false,
            comeback: false,
            uncontested_win: false,
        }
    }

    pub(super) fn capture(&mut self, cards: &[QiGuiCard]) {
        for card in cards {
            match card.rank() {
                QiGuiRank::Five => self.captured_fives += 1,
                QiGuiRank::Ten | QiGuiRank::King => self.captured_tens_and_kings += 1,
                _ => {}
            }
        }
    }
}
