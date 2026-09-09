use super::{
    BombKind, ClassifiedPlay, PlayError, QiGuiPlayKind, is_heaven_bomb, rank_counts,
    ranks_are_consecutive,
};
use crate::{QiGuiCard, QiGuiRuleSet};
use std::collections::HashSet;

pub fn classify(cards: &[QiGuiCard], rules: &QiGuiRuleSet) -> Result<ClassifiedPlay, PlayError> {
    if cards.is_empty() {
        return Err(PlayError::Empty);
    }
    validate_physical_cards(cards, rules)?;

    let mut sorted_cards = cards.to_vec();
    sorted_cards.sort_by(QiGuiCard::display_cmp);
    let rank_counts = rank_counts(&sorted_cards);
    let card_count = sorted_cards.len();
    let kind = if is_heaven_bomb(&rank_counts, card_count) {
        QiGuiPlayKind::HeavenBomb
    } else if rank_counts.len() == 1 && card_count >= 4 {
        QiGuiPlayKind::Bomb(BombKind::OfAKind {
            card_count,
            rank: sorted_cards[0].rank(),
        })
    } else if card_count == 1 {
        QiGuiPlayKind::Single
    } else if card_count == 2 && rank_counts.len() == 1 {
        QiGuiPlayKind::Pair
    } else if card_count == 3 && rank_counts.len() == 1 {
        QiGuiPlayKind::Triple
    } else if rules.advanced_play_types
        && card_count == 4
        && rank_counts.len() == 2
        && rank_counts.values().any(|count| *count == 3)
        && rank_counts.values().any(|count| *count == 1)
    {
        QiGuiPlayKind::TripleWithSingle
    } else if rules.advanced_play_types
        && card_count == 5
        && rank_counts.len() == 2
        && rank_counts.values().any(|count| *count == 3)
        && rank_counts.values().any(|count| *count == 2)
    {
        QiGuiPlayKind::TripleWithPair
    } else if card_count >= 6
        && card_count.is_multiple_of(3)
        && rank_counts.values().all(|count| *count == 3)
        && ranks_are_consecutive(rank_counts.keys().copied())
    {
        QiGuiPlayKind::Airplane {
            triple_count: card_count / 3,
        }
    } else if card_count >= 3
        && rank_counts.values().all(|count| *count == 1)
        && ranks_are_consecutive(rank_counts.keys().copied())
    {
        QiGuiPlayKind::Straight { card_count }
    } else if card_count >= 4
        && card_count.is_multiple_of(2)
        && rank_counts.values().all(|count| *count == 2)
        && ranks_are_consecutive(rank_counts.keys().copied())
    {
        QiGuiPlayKind::ConsecutivePairs {
            pair_count: card_count / 2,
        }
    } else {
        return Err(PlayError::InvalidPattern);
    };
    Ok(ClassifiedPlay {
        cards: sorted_cards,
        kind,
    })
}

fn validate_physical_cards(cards: &[QiGuiCard], rules: &QiGuiRuleSet) -> Result<(), PlayError> {
    let mut physical_cards = HashSet::with_capacity(cards.len());
    for &card in cards {
        #[cfg(not(feature = "developer"))]
        if card.deck() >= rules.deck_count {
            return Err(PlayError::CardOutsideConfiguredDeck(card));
        }
        if !physical_cards.insert(card) {
            return Err(PlayError::DuplicatePhysicalCard(card));
        }
    }
    #[cfg(feature = "developer")]
    let _ = rules;
    Ok(())
}
