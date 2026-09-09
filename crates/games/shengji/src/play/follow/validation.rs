use super::super::{
    Component, StructureDemands, category, classify_cards, classify_mixed_discard,
    component_can_be_beaten, component_kind_order, pair_units, satisfies_follow_structure,
    titanic_tops, tractor_tops, triple_units, validate_owned,
};
use super::{best_follow_tier, special_follow_hierarchy};
use crate::{
    FollowError, PlayError, ShengjiCard, ShengjiClassifiedPlay, ShengjiRank, ShengjiRuleSet,
    ShengjiSuit, ShengjiTrump, ThrowFailure, TrickPlay,
};
use std::collections::BTreeMap;

pub fn classify_lead(
    cards: &[ShengjiCard],
    trump: ShengjiTrump,
    rules: &ShengjiRuleSet,
    opponent_hands: &[&[ShengjiCard]],
) -> Result<TrickPlay, PlayError> {
    let attempted = classify_cards(cards, trump)?;
    if !attempted.is_throw() {
        return Ok(TrickPlay::Accepted(attempted));
    }
    if !rules.allow_throw {
        return Err(PlayError::ThrowDisabled);
    }

    let failed = attempted
        .components
        .iter()
        .filter(|component| {
            opponent_hands
                .iter()
                .any(|hand| component_can_be_beaten(component, attempted.category, hand, trump))
        })
        .min_by_key(|component| {
            (
                component.card_count(),
                component.comparison_strength(),
                component_kind_order(component),
            )
        });
    let Some(failed) = failed else {
        return Ok(TrickPlay::Accepted(attempted));
    };
    let forced = classify_cards(&failed.cards(), trump)?;
    Ok(TrickPlay::ThrowFailed(ThrowFailure {
        penalty_points: (cards.len() as u16).saturating_mul(rules.throw_penalty.points_per_card()),
        attempted,
        forced,
    }))
}

pub fn validate_follow(
    hand: &[ShengjiCard],
    cards: &[ShengjiCard],
    lead: &ShengjiClassifiedPlay,
    trump: ShengjiTrump,
) -> Result<ShengjiClassifiedPlay, FollowError> {
    if cards.len() != lead.cards.len() {
        return Err(FollowError::WrongCardCount {
            expected: lead.cards.len(),
            actual: cards.len(),
        });
    }
    validate_owned(cards, hand).map_err(FollowError::Play)?;
    let required_in_hand = hand
        .iter()
        .filter(|card| category(**card, trump) == lead.category)
        .count();
    let required = required_in_hand.min(cards.len());
    let actual = cards
        .iter()
        .filter(|card| category(**card, trump) == lead.category)
        .count();
    if actual != required {
        return Err(FollowError::MustFollowCategory { required, actual });
    }
    if required_in_hand >= cards.len() {
        let hand_cards = hand
            .iter()
            .copied()
            .filter(|card| category(*card, trump) == lead.category)
            .collect::<Vec<_>>();
        let proposed = cards.to_vec();
        let follows_structure = if let Some(hierarchy) = special_follow_hierarchy(lead) {
            best_follow_tier(&proposed, trump, &hierarchy, true)
                == best_follow_tier(&hand_cards, trump, &hierarchy, false)
        } else {
            match lead.components.as_slice() {
                [Component::Triple { .. }] => {
                    triple_follow_rank(&proposed) == triple_follow_rank(&hand_cards)
                }
                [Component::Titanic { triple_count, .. }] => {
                    titanic_follow_rank(&proposed, usize::from(*triple_count), trump)
                        == titanic_follow_rank(&hand_cards, usize::from(*triple_count), trump)
                }
                _ => {
                    let demands = StructureDemands::from_play(lead);
                    satisfies_follow_structure(
                        &hand_cards,
                        &proposed,
                        &demands,
                        trump,
                        lead.category,
                    )
                }
            }
        };
        if !follows_structure {
            return Err(FollowError::MustFollowStructure);
        }
    }
    if cards
        .iter()
        .all(|card| category(*card, trump) == category(cards[0], trump))
    {
        classify_cards(cards, trump).map_err(FollowError::Play)
    } else {
        Ok(classify_mixed_discard(cards, trump))
    }
}

pub(super) fn triple_follow_rank(cards: &[ShengjiCard]) -> u8 {
    let counts = face_counts(cards);
    if counts.values().any(|count| *count >= 3) {
        2
    } else if counts.values().any(|count| *count >= 2) {
        1
    } else {
        0
    }
}

/// 泰坦尼克跟牌顺序：泰坦尼克 > 拖拉机 > 尽可能多三同张 > 尽可能多对子。
/// 对六张牌展开后正好对应规则列出的八个层级。
fn titanic_follow_rank(
    cards: &[ShengjiCard],
    triple_count: usize,
    trump: ShengjiTrump,
) -> (bool, bool, usize, usize) {
    let triples = triple_units(cards, trump);
    let pairs = pair_units(cards, trump);
    let full_titanic = !titanic_tops(&triples, triple_count).is_empty();
    let full_tractor = !tractor_tops(&pairs, triple_count).is_empty();
    if full_titanic {
        return (true, false, 0, 0);
    }
    if full_tractor {
        return (false, true, 0, 0);
    }
    let required_cards = triple_count * 3;
    let selected_triples = triples.len().min(triple_count);
    let remaining_slots = required_cards.saturating_sub(selected_triples * 3);
    let remaining_pair_faces = pairs.len().saturating_sub(selected_triples);
    let selected_pairs = remaining_pair_faces.min(remaining_slots / 2);
    (full_titanic, full_tractor, selected_triples, selected_pairs)
}

fn face_counts(cards: &[ShengjiCard]) -> BTreeMap<(Option<ShengjiSuit>, ShengjiRank), usize> {
    let mut counts = BTreeMap::new();
    for card in cards {
        *counts.entry((card.suit(), card.rank())).or_default() += 1;
    }
    counts
}
