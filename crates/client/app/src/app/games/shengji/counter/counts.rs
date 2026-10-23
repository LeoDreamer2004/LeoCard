use leocard_protocol::ShengjiSnapshot;
use leocard_shengji::{ShengjiCard, ShengjiRank, ShengjiSuit, ShengjiTrump};
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub(super) struct CounterCell {
    pub card: ShengjiCard,
    pub remaining: u8,
}

#[derive(Default)]
pub(super) struct CardCounts {
    excluded: HashSet<ShengjiCard>,
    deck_count: u8,
}

impl CardCounts {
    pub fn update(&mut self, game: &ShengjiSnapshot) {
        self.deck_count = game.rules.deck_count;
        self.excluded.clear();
        self.excluded.extend(
            game.your_hand
                .iter()
                .chain(&game.played_cards)
                .chain(&game.your_buried)
                .copied(),
        );
    }

    fn cell(&self, card: ShengjiCard) -> CounterCell {
        let known = self
            .excluded
            .iter()
            .filter(|known| known.same_face(card))
            .count();
        CounterCell {
            card,
            remaining: self.deck_count.saturating_sub(known as u8),
        }
    }

    pub fn rows(&self, trump: ShengjiTrump) -> [[Option<CounterCell>; 14]; 4] {
        let mut suits = ShengjiSuit::ALL;
        // 无主局仍按黑、红、梅、方排列，有主局将主花色放在首行。
        suits.reverse();
        if let Some(suit) = trump.suit {
            let position = suits
                .iter()
                .position(|candidate| *candidate == suit)
                .unwrap();
            suits.swap(0, position);
        }
        let mut rows = [[None; 14]; 4];
        rows[0][0] = Some(self.cell(ShengjiCard::big_joker(0)));
        rows[1][0] = Some(self.cell(ShengjiCard::small_joker(0)));
        for (row, suit) in rows.iter_mut().zip(suits) {
            row[1] = Some(self.cell(ShengjiCard::suited(0, suit, trump.level)));
            let constant = trump.constant_trump && trump.level != ShengjiRank::Two;
            let mut column = 2;
            if constant {
                row[column] = Some(self.cell(ShengjiCard::suited(0, suit, ShengjiRank::Two)));
                column += 1;
            }
            for rank in ShengjiRank::LEVELS.into_iter().rev() {
                if rank == trump.level || (constant && rank == ShengjiRank::Two) {
                    continue;
                }
                row[column] = Some(self.cell(ShengjiCard::suited(0, suit, rank)));
                column += 1;
            }
        }
        rows
    }
}
