use super::MahjongBotObservation;
use crate::{MahjongMeldKind, MahjongTileKind};

impl MahjongBotObservation {
    pub(super) fn danger(&self, index: usize) -> f64 {
        let tile = MahjongTileKind::from_index34(index).expect("tile index");
        let mut danger = 0.0;
        for other in self
            .public
            .iter()
            .filter(|state| state.id != self.player && !state.dead_hand)
        {
            let river: Vec<_> = self
                .discards
                .iter()
                .filter(|discard| discard.player == other.id)
                .collect();
            let progress = (river.len() as f64 / 13.0).min(1.0);
            let open = other
                .melds
                .iter()
                .filter(|meld| meld.tile.is_some())
                .count();
            let mut risk = (0.04 + progress * 0.11 + open as f64 * 0.07).min(0.5);
            // Recently discarded tiles are less attractive, never absolutely safe in MCR.
            if river
                .iter()
                .rev()
                .take(3)
                .any(|discard| discard.tile.kind() == tile)
            {
                risk *= 0.45;
            }
            let held = self.hand.iter().filter(|held| held.kind() == tile).count() as u8;
            let outside = 4u8.saturating_sub(self.unseen[index] + held);
            if tile.is_honor() {
                risk *= if outside >= 2 { 0.18 } else { 0.6 };
            } else if index.is_multiple_of(9) || index % 9 == 8 {
                risk *= 0.7;
            }
            let suits: Vec<_> = other
                .melds
                .iter()
                .filter_map(|meld| match meld.tile {
                    Some(MahjongTileKind::Suited { suit, .. }) => Some(suit),
                    _ => None,
                })
                .collect();
            if suits.len() >= 2 && suits.iter().all(|suit| *suit == suits[0]) {
                risk *= match tile {
                    MahjongTileKind::Suited { suit, .. } if suit != suits[0] => 0.35,
                    _ => 1.6,
                };
            }
            if other.melds.len() == 4 && outside >= 3 {
                risk *= 0.08;
            }
            let value = 12.0
                + open as f64 * 3.0
                + if other
                    .melds
                    .iter()
                    .all(|meld| meld.kind != MahjongMeldKind::Chow)
                {
                    3.0
                } else {
                    0.0
                };
            danger += risk * value;
        }
        let ahead = self
            .scores
            .iter()
            .all(|score| *score <= self.scores[self.player.index()]);
        danger * if self.wall < 20 && ahead { 2.0 } else { 1.0 }
    }
}
