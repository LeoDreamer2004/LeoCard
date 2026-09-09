use crate::{QiGuiCard, QiGuiRank};
use std::collections::HashMap;

pub(super) fn rank_counts(cards: &[QiGuiCard]) -> HashMap<QiGuiRank, usize> {
    let mut result = HashMap::new();
    for card in cards {
        *result.entry(card.rank()).or_insert(0) += 1;
    }
    result
}

pub(super) fn is_heaven_bomb(rank_counts: &HashMap<QiGuiRank, usize>, card_count: usize) -> bool {
    const REQUIRED_RANKS: [QiGuiRank; 5] = [
        QiGuiRank::Seven,
        QiGuiRank::Joker,
        QiGuiRank::Five,
        QiGuiRank::Two,
        QiGuiRank::Three,
    ];
    card_count == REQUIRED_RANKS.len()
        && rank_counts.len() == REQUIRED_RANKS.len()
        && rank_counts.values().all(|count| *count == 1)
        && REQUIRED_RANKS
            .into_iter()
            .all(|rank| rank_counts.contains_key(&rank))
}

pub(super) fn ranks_are_consecutive(ranks: impl IntoIterator<Item = QiGuiRank>) -> bool {
    let mut strengths: Vec<_> = ranks.into_iter().map(QiGuiRank::strength).collect();
    strengths.sort_unstable();
    strengths
        .windows(2)
        .all(|window| window[1] == window[0] + 1)
}
