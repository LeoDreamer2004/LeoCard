use leocard_guandan::{GuandanCard, GuandanPlayKind, classify, rank_strength};
use std::cmp::Reverse;
use std::collections::HashSet;

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub(super) enum HandLayoutMode {
    #[default]
    Columns,
    Row,
}

#[derive(Default)]
pub(super) struct HandArrangement {
    pub mode: HandLayoutMode,
    groups: Vec<Vec<GuandanCard>>,
}

pub(super) struct HandColumn {
    pub cards: Vec<GuandanCard>,
    pub arranged: bool,
}

impl HandArrangement {
    pub fn toggle_layout(&mut self) {
        self.mode = match self.mode {
            HandLayoutMode::Columns => HandLayoutMode::Row,
            HandLayoutMode::Row => HandLayoutMode::Columns,
        };
    }

    pub fn has_groups(&self) -> bool {
        !self.groups.is_empty()
    }

    pub fn reset_groups(&mut self) {
        self.groups.clear();
    }

    pub fn reconcile(&mut self, hand: &[GuandanCard], level: u8) {
        self.groups.retain_mut(|group| {
            let previous = group.len();
            group.retain(|card| hand.contains(card));
            group.len() >= 2 && (group.len() == previous || classify(group, level).is_ok())
        });
    }

    pub fn can_group(
        &self,
        selected: &HashSet<GuandanCard>,
        hand: &[GuandanCard],
        level: u8,
    ) -> bool {
        selected.len() >= 2
            && selected.iter().all(|card| hand.contains(card))
            && !self.groups.iter().any(|group| {
                group.len() == selected.len() && group.iter().all(|card| selected.contains(card))
            })
            && classify(&selected.iter().copied().collect::<Vec<_>>(), level).is_ok()
    }

    pub fn group_selected(
        &mut self,
        selected: &HashSet<GuandanCard>,
        hand: &[GuandanCard],
        level: u8,
    ) -> bool {
        if !self.can_group(selected, hand, level) {
            return false;
        }
        self.groups.retain_mut(|group| {
            group.retain(|card| !selected.contains(card));
            group.len() >= 2 && classify(group, level).is_ok()
        });
        let mut cards = selected.iter().copied().collect::<Vec<_>>();
        let play = classify(&cards, level).expect("group selection was checked");
        if matches!(
            play.kind,
            GuandanPlayKind::Straight
                | GuandanPlayKind::StraightFlush
                | GuandanPlayKind::ConsecutivePairs
                | GuandanPlayKind::ConsecutiveTriples
        ) {
            cards.sort_by_key(|card| {
                (
                    card.is_wild(level),
                    Reverse((
                        if play.value == 5 && card.rank == 14 {
                            1
                        } else {
                            card.rank
                        },
                        card.suit,
                        card.deck,
                    )),
                )
            });
        } else {
            cards = sorted_cards(&cards, level);
        }
        self.groups.push(cards);
        true
    }

    pub fn columns(&self, hand: &[GuandanCard], level: u8) -> Vec<HandColumn> {
        let grouped: HashSet<_> = self.groups.iter().flatten().copied().collect();
        let mut columns: Vec<HandColumn> = Vec::new();
        for card in sorted_cards(hand, level) {
            if grouped.contains(&card) {
                continue;
            }
            if let Some(column) = columns.last_mut()
                && column.cards[0].rank == card.rank
            {
                column.cards.push(card);
            } else {
                columns.push(HandColumn {
                    cards: vec![card],
                    arranged: false,
                });
            }
        }
        columns.extend(self.groups.iter().cloned().map(|cards| HandColumn {
            cards,
            arranged: true,
        }));
        columns
    }
}

pub(super) fn sorted_cards(hand: &[GuandanCard], level: u8) -> Vec<GuandanCard> {
    let mut cards = hand.to_vec();
    cards.sort_by_key(|card| {
        Reverse((
            rank_strength(card.rank, level),
            card.is_wild(level),
            card.suit,
            card.deck,
        ))
    });
    cards
}
