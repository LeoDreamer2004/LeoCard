use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::{Card, Rank, RuleSet, SameCardPolicy, SuitComparison};

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BombKind {
    OfAKind { card_count: usize, rank: Rank },
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PlayKind {
    Single,
    Pair,
    Straight {
        card_count: usize,
    },
    ConsecutivePairs {
        pair_count: usize,
    },
    Triple,
    /// 一组三张相同点数的主体，附带一张任意其他点数的牌。
    TripleWithSingle,
    /// 一组三张相同点数的主体，附带一对其他点数的牌。
    TripleWithPair,
    /// 至少两组按牌力连续的三张牌，例如 666777。
    Airplane {
        triple_count: usize,
    },
    Bomb(BombKind),
    /// 恰好由 7、王、5、2、3 各一张组成，能够压过其他所有牌型。
    HeavenBomb,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassifiedPlay {
    cards: Vec<Card>,
    kind: PlayKind,
}

impl ClassifiedPlay {
    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    pub fn kind(&self) -> &PlayKind {
        &self.kind
    }

    pub fn score(&self) -> u16 {
        self.cards.iter().map(|card| card.score()).sum()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayComparison {
    Greater,
    Equivalent,
    Lower,
    Incompatible,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayError {
    Empty,
    DuplicatePhysicalCard(Card),
    CardOutsideConfiguredDeck(Card),
    InvalidPattern,
}

impl fmt::Display for PlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("出牌不能为空"),
            Self::DuplicatePhysicalCard(card) => write!(f, "同一张物理牌被重复提交：{card}"),
            Self::CardOutsideConfiguredDeck(card) => write!(f, "牌不属于当前配置的牌堆：{card}"),
            Self::InvalidPattern => f.write_str("这些牌不能组成合法牌型"),
        }
    }
}

impl std::error::Error for PlayError {}

pub fn classify(cards: &[Card], rules: &RuleSet) -> Result<ClassifiedPlay, PlayError> {
    if cards.is_empty() {
        return Err(PlayError::Empty);
    }

    validate_physical_cards(cards, rules)?;

    let mut sorted_cards = cards.to_vec();
    sorted_cards.sort_by(Card::display_cmp);

    let rank_counts = rank_counts(&sorted_cards);
    let card_count = sorted_cards.len();

    let kind = if is_heaven_bomb(&rank_counts, card_count) {
        PlayKind::HeavenBomb
    } else if rank_counts.len() == 1 && card_count >= 4 {
        PlayKind::Bomb(BombKind::OfAKind {
            card_count,
            rank: sorted_cards[0].rank(),
        })
    } else if card_count == 1 {
        PlayKind::Single
    } else if card_count == 2 && rank_counts.len() == 1 {
        PlayKind::Pair
    } else if card_count == 3 && rank_counts.len() == 1 {
        PlayKind::Triple
    } else if rules.advanced_play_types
        && card_count == 4
        && rank_counts.len() == 2
        && rank_counts.values().any(|count| *count == 3)
        && rank_counts.values().any(|count| *count == 1)
    {
        PlayKind::TripleWithSingle
    } else if rules.advanced_play_types
        && card_count == 5
        && rank_counts.len() == 2
        && rank_counts.values().any(|count| *count == 3)
        && rank_counts.values().any(|count| *count == 2)
    {
        PlayKind::TripleWithPair
    } else if card_count >= 6
        && card_count % 3 == 0
        && rank_counts.values().all(|count| *count == 3)
        && ranks_are_consecutive(rank_counts.keys().copied())
    {
        PlayKind::Airplane {
            triple_count: card_count / 3,
        }
    } else if card_count >= 3
        && rank_counts.values().all(|count| *count == 1)
        && ranks_are_consecutive(rank_counts.keys().copied())
    {
        PlayKind::Straight { card_count }
    } else if card_count >= 4
        && card_count % 2 == 0
        && rank_counts.values().all(|count| *count == 2)
        && ranks_are_consecutive(rank_counts.keys().copied())
    {
        PlayKind::ConsecutivePairs {
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

fn validate_physical_cards(cards: &[Card], rules: &RuleSet) -> Result<(), PlayError> {
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

pub fn compare_plays(
    challenger: &ClassifiedPlay,
    current: &ClassifiedPlay,
    rules: &RuleSet,
) -> PlayComparison {
    use BombKind::OfAKind;
    use PlayKind::{Bomb, HeavenBomb};

    if rules.advanced_play_types {
        let comparison = advanced_type_comparison(challenger.kind(), current.kind());
        if comparison != PlayComparison::Incompatible {
            return comparison;
        }
    }

    match (challenger.kind(), current.kind()) {
        (HeavenBomb, HeavenBomb) => {
            semantic_comparison(challenger.cards(), current.cards(), rules.suit_comparison)
        }
        (HeavenBomb, _) => PlayComparison::Greater,
        (_, HeavenBomb) => PlayComparison::Lower,
        (Bomb(OfAKind { .. }), kind) if !matches!(kind, Bomb(_)) => PlayComparison::Greater,
        (kind, Bomb(OfAKind { .. })) if !matches!(kind, Bomb(_)) => PlayComparison::Lower,
        (
            Bomb(OfAKind {
                card_count: challenger_count,
                ..
            }),
            Bomb(OfAKind {
                card_count: current_count,
                ..
            }),
        ) => match challenger_count.cmp(current_count) {
            Ordering::Greater => PlayComparison::Greater,
            Ordering::Less => PlayComparison::Lower,
            Ordering::Equal => {
                semantic_comparison(challenger.cards(), current.cards(), rules.suit_comparison)
            }
        },
        (PlayKind::TripleWithSingle, PlayKind::TripleWithSingle)
        | (PlayKind::TripleWithPair, PlayKind::TripleWithPair) => {
            compare_triple_components(challenger.cards(), current.cards(), rules.suit_comparison)
        }
        (challenger_kind, current_kind) if same_non_bomb_shape(challenger_kind, current_kind) => {
            semantic_comparison(challenger.cards(), current.cards(), rules.suit_comparison)
        }
        _ => PlayComparison::Incompatible,
    }
}

fn advanced_type_comparison(challenger: &PlayKind, current: &PlayKind) -> PlayComparison {
    match (challenger, current) {
        (PlayKind::Triple, PlayKind::Straight { card_count: 3 })
        | (PlayKind::ConsecutivePairs { pair_count: 2 }, PlayKind::Straight { card_count: 4 })
        | (PlayKind::Airplane { triple_count: 2 }, PlayKind::ConsecutivePairs { pair_count: 3 }) => {
            PlayComparison::Greater
        }
        (PlayKind::Straight { card_count: 3 }, PlayKind::Triple)
        | (PlayKind::Straight { card_count: 4 }, PlayKind::ConsecutivePairs { pair_count: 2 })
        | (PlayKind::ConsecutivePairs { pair_count: 3 }, PlayKind::Airplane { triple_count: 2 }) => {
            PlayComparison::Lower
        }
        _ => PlayComparison::Incompatible,
    }
}

fn compare_triple_components(
    challenger: &[Card],
    current: &[Card],
    suit_comparison: SuitComparison,
) -> PlayComparison {
    let challenger = triple_component(challenger);
    let current = triple_component(current);
    semantic_comparison(&challenger, &current, suit_comparison)
}

fn triple_component(cards: &[Card]) -> Vec<Card> {
    let counts = rank_counts(cards);
    let triple_rank = counts
        .into_iter()
        .find_map(|(rank, count)| (count == 3).then_some(rank))
        .expect("triple-carry play always contains exactly one triple component");
    cards
        .iter()
        .copied()
        .filter(|card| card.rank() == triple_rank)
        .collect()
}

pub fn can_beat(challenger: &ClassifiedPlay, current: &ClassifiedPlay, rules: &RuleSet) -> bool {
    match compare_plays(challenger, current, rules) {
        PlayComparison::Greater => true,
        PlayComparison::Equivalent => rules.same_card_policy == SameCardPolicy::CanFollow,
        PlayComparison::Lower | PlayComparison::Incompatible => false,
    }
}

fn rank_counts(cards: &[Card]) -> HashMap<Rank, usize> {
    let mut result = HashMap::new();
    for card in cards {
        *result.entry(card.rank()).or_insert(0) += 1;
    }
    result
}

fn is_heaven_bomb(rank_counts: &HashMap<Rank, usize>, card_count: usize) -> bool {
    const REQUIRED_RANKS: [Rank; 5] =
        [Rank::Seven, Rank::Joker, Rank::Five, Rank::Two, Rank::Three];
    card_count == REQUIRED_RANKS.len()
        && rank_counts.len() == REQUIRED_RANKS.len()
        && rank_counts.values().all(|count| *count == 1)
        && REQUIRED_RANKS
            .into_iter()
            .all(|rank| rank_counts.contains_key(&rank))
}

fn ranks_are_consecutive(ranks: impl IntoIterator<Item = Rank>) -> bool {
    let mut strengths: Vec<_> = ranks.into_iter().map(Rank::strength).collect();
    strengths.sort_unstable();
    strengths
        .windows(2)
        .all(|window| window[1] == window[0] + 1)
}

fn same_non_bomb_shape(left: &PlayKind, right: &PlayKind) -> bool {
    match (left, right) {
        (PlayKind::Single, PlayKind::Single)
        | (PlayKind::Pair, PlayKind::Pair)
        | (PlayKind::Triple, PlayKind::Triple) => true,
        (
            PlayKind::Straight {
                card_count: left_count,
            },
            PlayKind::Straight {
                card_count: right_count,
            },
        ) => left_count == right_count,
        (
            PlayKind::ConsecutivePairs {
                pair_count: left_count,
            },
            PlayKind::ConsecutivePairs {
                pair_count: right_count,
            },
        ) => left_count == right_count,
        (
            PlayKind::Airplane {
                triple_count: left_count,
            },
            PlayKind::Airplane {
                triple_count: right_count,
            },
        ) => left_count == right_count,
        _ => false,
    }
}

fn semantic_comparison(
    left: &[Card],
    right: &[Card],
    suit_comparison: SuitComparison,
) -> PlayComparison {
    let mut left_ranks: Vec<_> = left.iter().map(|card| card.rank().strength()).collect();
    let mut right_ranks: Vec<_> = right.iter().map(|card| card.rank().strength()).collect();
    left_ranks.sort_unstable_by(|a, b| b.cmp(a));
    right_ranks.sort_unstable_by(|a, b| b.cmp(a));

    let ordering = left_ranks
        .cmp(&right_ranks)
        .then_with(|| match suit_comparison {
            SuitComparison::HighestCard => left
                .iter()
                .map(|card| card.semantic_strength())
                .max()
                .cmp(&right.iter().map(|card| card.semantic_strength()).max()),
            SuitComparison::Lexicographic => {
                let mut left_strengths: Vec<_> =
                    left.iter().map(|card| card.semantic_strength()).collect();
                let mut right_strengths: Vec<_> =
                    right.iter().map(|card| card.semantic_strength()).collect();
                left_strengths.sort_unstable_by(|a, b| b.cmp(a));
                right_strengths.sort_unstable_by(|a, b| b.cmp(a));
                left_strengths.cmp(&right_strengths)
            }
            SuitComparison::SumPoints => left
                .iter()
                .map(|card| card.suit_points())
                .sum::<u16>()
                .cmp(&right.iter().map(|card| card.suit_points()).sum::<u16>()),
        });

    match ordering {
        Ordering::Greater => PlayComparison::Greater,
        Ordering::Equal => PlayComparison::Equivalent,
        Ordering::Less => PlayComparison::Lower,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Suit::{Club, Diamond, Heart, Spade};
    use crate::SuitComparison;

    fn card(rank: Rank) -> Card {
        Card::suited(0, Diamond, rank)
    }

    #[test]
    fn unusual_rank_order_drives_straights() {
        let rules = RuleSet::default();
        let four_six_eight_nine = [
            card(Rank::Four),
            card(Rank::Six),
            card(Rank::Eight),
            card(Rank::Nine),
        ];
        let queen_to_two = [
            card(Rank::Queen),
            card(Rank::King),
            card(Rank::Ace),
            card(Rank::Three),
            card(Rank::Two),
        ];
        let ordinary_456 = [card(Rank::Four), card(Rank::Five), card(Rank::Six)];

        assert!(matches!(
            classify(&four_six_eight_nine, &rules).unwrap().kind(),
            PlayKind::Straight { card_count: 4 }
        ));
        assert!(matches!(
            classify(&queen_to_two, &rules).unwrap().kind(),
            PlayKind::Straight { card_count: 5 }
        ));
        assert_eq!(
            classify(&ordinary_456, &rules),
            Err(PlayError::InvalidPattern)
        );
    }

    #[test]
    fn recognizes_consecutive_pairs() {
        let rules = RuleSet {
            deck_count: 2,
            ..RuleSet::default()
        };
        let cards = [
            Card::suited(0, Diamond, Rank::Queen),
            Card::suited(0, Club, Rank::Queen),
            Card::suited(0, Diamond, Rank::King),
            Card::suited(0, Club, Rank::King),
        ];
        assert!(matches!(
            classify(&cards, &rules).unwrap().kind(),
            PlayKind::ConsecutivePairs { pair_count: 2 }
        ));
    }

    #[test]
    fn bomb_count_wins_before_rank_and_bombs_override_shapes() {
        let rules = RuleSet {
            deck_count: 2,
            ..RuleSet::default()
        };
        let four_sevens = classify(
            &[
                Card::suited(0, Diamond, Rank::Seven),
                Card::suited(0, Club, Rank::Seven),
                Card::suited(0, Heart, Rank::Seven),
                Card::suited(0, Spade, Rank::Seven),
            ],
            &rules,
        )
        .unwrap();
        let five_fours = classify(
            &[
                Card::suited(0, Diamond, Rank::Four),
                Card::suited(0, Club, Rank::Four),
                Card::suited(0, Heart, Rank::Four),
                Card::suited(0, Spade, Rank::Four),
                Card::suited(1, Diamond, Rank::Four),
            ],
            &rules,
        )
        .unwrap();
        let pair = classify(
            &[
                Card::suited(0, Diamond, Rank::Seven),
                Card::suited(0, Club, Rank::Seven),
            ],
            &rules,
        )
        .unwrap();
        let long_straight = classify(
            &[
                Card::suited(0, Diamond, Rank::Four),
                Card::suited(0, Diamond, Rank::Six),
                Card::suited(0, Diamond, Rank::Eight),
                Card::suited(0, Diamond, Rank::Nine),
                Card::suited(0, Diamond, Rank::Ten),
                Card::suited(0, Diamond, Rank::Jack),
            ],
            &rules,
        )
        .unwrap();
        let long_consecutive_pairs = classify(
            &[
                Card::suited(0, Diamond, Rank::Four),
                Card::suited(0, Club, Rank::Four),
                Card::suited(0, Diamond, Rank::Six),
                Card::suited(0, Club, Rank::Six),
                Card::suited(0, Diamond, Rank::Eight),
                Card::suited(0, Club, Rank::Eight),
            ],
            &rules,
        )
        .unwrap();

        assert_eq!(
            compare_plays(&five_fours, &four_sevens, &rules),
            PlayComparison::Greater
        );
        assert_eq!(
            compare_plays(&four_sevens, &pair, &rules),
            PlayComparison::Greater
        );
        assert_eq!(
            compare_plays(&four_sevens, &long_straight, &rules),
            PlayComparison::Greater
        );
        assert_eq!(
            compare_plays(&four_sevens, &long_consecutive_pairs, &rules),
            PlayComparison::Greater
        );
    }

    #[test]
    fn jokers_share_a_rank_and_four_jokers_are_a_normal_bomb() {
        let one_deck = RuleSet::default();
        let joker_pair = classify(
            &[
                Card::suited(0, Club, Rank::Joker),
                Card::suited(0, Spade, Rank::Joker),
            ],
            &one_deck,
        )
        .unwrap();
        assert_eq!(joker_pair.kind(), &PlayKind::Pair);

        let two_decks = RuleSet {
            deck_count: 2,
            ..RuleSet::default()
        };
        let four_jokers = classify(
            &[
                Card::suited(0, Club, Rank::Joker),
                Card::suited(0, Spade, Rank::Joker),
                Card::suited(1, Club, Rank::Joker),
                Card::suited(1, Spade, Rank::Joker),
            ],
            &two_decks,
        )
        .unwrap();
        let four_sevens = classify(
            &[
                Card::suited(0, Diamond, Rank::Seven),
                Card::suited(0, Club, Rank::Seven),
                Card::suited(0, Heart, Rank::Seven),
                Card::suited(0, Spade, Rank::Seven),
            ],
            &two_decks,
        )
        .unwrap();

        assert!(matches!(
            four_jokers.kind(),
            PlayKind::Bomb(BombKind::OfAKind {
                card_count: 4,
                rank: Rank::Joker
            })
        ));
        assert_eq!(
            compare_plays(&four_jokers, &four_sevens, &two_decks),
            PlayComparison::Lower
        );
    }

    #[cfg(feature = "developer")]
    #[test]
    fn developer_classification_allows_copies_beyond_the_configured_deck_count() {
        let one_deck = RuleSet::default();
        let impossible_small_joker_pair = [
            Card::suited(0, Club, Rank::Joker),
            Card::suited(1, Club, Rank::Joker),
        ];

        assert_eq!(
            classify(&impossible_small_joker_pair, &one_deck)
                .unwrap()
                .kind(),
            &PlayKind::Pair
        );
    }

    #[cfg(not(feature = "developer"))]
    #[test]
    fn normal_classification_rejects_copies_beyond_the_configured_deck_count() {
        let one_deck = RuleSet::default();
        let outside_deck = Card::suited(1, Club, Rank::Joker);

        assert_eq!(
            classify(&[outside_deck], &one_deck),
            Err(PlayError::CardOutsideConfiguredDeck(outside_deck))
        );
    }

    #[test]
    fn either_joker_can_fill_the_same_straight_position() {
        let rules = RuleSet::default();
        for joker_suit in [Club, Spade] {
            let straight = classify(
                &[
                    Card::suited(0, Diamond, Rank::Five),
                    Card::suited(0, joker_suit, Rank::Joker),
                    Card::suited(0, Diamond, Rank::Seven),
                ],
                &rules,
            )
            .unwrap();
            assert_eq!(straight.kind(), &PlayKind::Straight { card_count: 3 });
        }

        assert_eq!(
            classify(
                &[
                    Card::suited(0, Diamond, Rank::Five),
                    Card::suited(0, Club, Rank::Joker),
                    Card::suited(0, Spade, Rank::Joker),
                    Card::suited(0, Diamond, Rank::Seven),
                ],
                &rules,
            ),
            Err(PlayError::InvalidPattern)
        );
    }

    #[test]
    fn exact_duplicate_strength_obeys_room_setting() {
        let strict = RuleSet {
            deck_count: 2,
            ..RuleSet::default()
        };
        let following = RuleSet {
            same_card_policy: SameCardPolicy::CanFollow,
            ..strict
        };
        let first = classify(&[Card::suited(0, Spade, Rank::Ace)], &strict).unwrap();
        let duplicate = classify(&[Card::suited(1, Spade, Rank::Ace)], &strict).unwrap();

        assert_eq!(
            compare_plays(&duplicate, &first, &strict),
            PlayComparison::Equivalent
        );
        assert!(!can_beat(&duplicate, &first, &strict));
        assert!(can_beat(&duplicate, &first, &following));
    }

    #[test]
    fn non_bombs_require_the_same_shape_and_length() {
        let rules = RuleSet::default();
        let pair = classify(
            &[
                Card::suited(0, Diamond, Rank::Four),
                Card::suited(0, Club, Rank::Four),
            ],
            &rules,
        )
        .unwrap();
        let triple = classify(
            &[
                Card::suited(0, Diamond, Rank::Six),
                Card::suited(0, Club, Rank::Six),
                Card::suited(0, Heart, Rank::Six),
            ],
            &rules,
        )
        .unwrap();
        let short_straight = classify(
            &[card(Rank::Four), card(Rank::Six), card(Rank::Eight)],
            &rules,
        )
        .unwrap();
        let long_straight = classify(
            &[
                card(Rank::Six),
                card(Rank::Eight),
                card(Rank::Nine),
                card(Rank::Ten),
            ],
            &rules,
        )
        .unwrap();

        assert_eq!(
            compare_plays(&triple, &pair, &rules),
            PlayComparison::Incompatible
        );
        assert_eq!(
            compare_plays(&long_straight, &short_straight, &rules),
            PlayComparison::Incompatible
        );
    }

    #[test]
    fn advanced_play_types_obey_the_three_fixed_suppression_relations() {
        let disabled = RuleSet::default();
        let enabled = RuleSet {
            advanced_play_types: true,
            ..disabled
        };
        let straight_three = classify(
            &[card(Rank::Eight), card(Rank::Nine), card(Rank::Ten)],
            &enabled,
        )
        .unwrap();
        let triple = classify(
            &[
                Card::suited(0, Diamond, Rank::Four),
                Card::suited(0, Club, Rank::Four),
                Card::suited(0, Heart, Rank::Four),
            ],
            &enabled,
        )
        .unwrap();
        let straight_four = classify(
            &[
                card(Rank::Eight),
                card(Rank::Nine),
                card(Rank::Ten),
                card(Rank::Jack),
            ],
            &enabled,
        )
        .unwrap();
        let two_pairs = classify(
            &[
                Card::suited(0, Diamond, Rank::Four),
                Card::suited(0, Club, Rank::Four),
                Card::suited(0, Diamond, Rank::Six),
                Card::suited(0, Club, Rank::Six),
            ],
            &enabled,
        )
        .unwrap();
        let three_pairs = classify(
            &[
                Card::suited(0, Diamond, Rank::Eight),
                Card::suited(0, Club, Rank::Eight),
                Card::suited(0, Diamond, Rank::Nine),
                Card::suited(0, Club, Rank::Nine),
                Card::suited(0, Diamond, Rank::Ten),
                Card::suited(0, Club, Rank::Ten),
            ],
            &enabled,
        )
        .unwrap();
        let two_plane = classify(
            &[
                Card::suited(0, Diamond, Rank::Four),
                Card::suited(0, Club, Rank::Four),
                Card::suited(0, Heart, Rank::Four),
                Card::suited(0, Diamond, Rank::Six),
                Card::suited(0, Club, Rank::Six),
                Card::suited(0, Heart, Rank::Six),
            ],
            &enabled,
        )
        .unwrap();

        assert_eq!(
            compare_plays(&triple, &straight_three, &disabled),
            PlayComparison::Incompatible
        );
        for (stronger, weaker) in [
            (&triple, &straight_three),
            (&two_pairs, &straight_four),
            (&two_plane, &three_pairs),
        ] {
            assert_eq!(
                compare_plays(stronger, weaker, &enabled),
                PlayComparison::Greater
            );
            assert_eq!(
                compare_plays(weaker, stronger, &enabled),
                PlayComparison::Lower
            );
            assert!(can_beat(stronger, weaker, &enabled));
            assert!(!can_beat(weaker, stronger, &enabled));
        }
    }

    #[test]
    fn advanced_triple_carries_compare_only_the_triple_component() {
        let disabled = RuleSet::default();
        let enabled = RuleSet {
            advanced_play_types: true,
            ..disabled
        };
        let triple_four = [
            Card::suited(0, Diamond, Rank::Four),
            Card::suited(0, Club, Rank::Four),
            Card::suited(0, Heart, Rank::Four),
        ];
        let low_kicker_cards = [
            triple_four[0],
            triple_four[1],
            triple_four[2],
            card(Rank::Six),
        ];
        let high_kicker_cards = [
            triple_four[0],
            triple_four[1],
            triple_four[2],
            card(Rank::Seven),
        ];
        assert_eq!(
            classify(&low_kicker_cards, &disabled),
            Err(PlayError::InvalidPattern)
        );
        let low_kicker = classify(&low_kicker_cards, &enabled).unwrap();
        let high_kicker = classify(&high_kicker_cards, &enabled).unwrap();
        assert_eq!(low_kicker.kind(), &PlayKind::TripleWithSingle);
        assert_eq!(
            compare_plays(&high_kicker, &low_kicker, &enabled),
            PlayComparison::Equivalent
        );
        assert!(!can_beat(&high_kicker, &low_kicker, &enabled));
        let following = RuleSet {
            same_card_policy: SameCardPolicy::CanFollow,
            ..enabled
        };
        assert!(can_beat(&high_kicker, &low_kicker, &following));

        let lower_triple_with_pair = classify(
            &[
                triple_four[0],
                triple_four[1],
                triple_four[2],
                Card::suited(0, Diamond, Rank::Seven),
                Card::suited(0, Club, Rank::Seven),
            ],
            &enabled,
        )
        .unwrap();
        let higher_triple_with_pair = classify(
            &[
                Card::suited(0, Diamond, Rank::Six),
                Card::suited(0, Club, Rank::Six),
                Card::suited(0, Heart, Rank::Six),
                Card::suited(0, Diamond, Rank::Four),
                Card::suited(0, Club, Rank::Four),
            ],
            &enabled,
        )
        .unwrap();
        assert_eq!(lower_triple_with_pair.kind(), &PlayKind::TripleWithPair);
        assert_eq!(
            compare_plays(&higher_triple_with_pair, &lower_triple_with_pair, &enabled),
            PlayComparison::Greater
        );

        let straight = classify(
            &[card(Rank::Eight), card(Rank::Nine), card(Rank::Ten)],
            &enabled,
        )
        .unwrap();
        assert_eq!(
            compare_plays(&high_kicker, &straight, &enabled),
            PlayComparison::Incompatible
        );
        assert!(!can_beat(&high_kicker, &straight, &enabled));
    }

    #[test]
    fn suit_breaks_a_same_rank_tie() {
        let rules = RuleSet::default();
        let diamond = classify(&[Card::suited(0, Diamond, Rank::Four)], &rules).unwrap();
        let spade = classify(&[Card::suited(0, Spade, Rank::Four)], &rules).unwrap();

        assert_eq!(
            compare_plays(&spade, &diamond, &rules),
            PlayComparison::Greater
        );
    }

    #[test]
    fn all_three_suit_comparison_modes_are_distinct() {
        let highest = RuleSet {
            deck_count: 2,
            suit_comparison: SuitComparison::HighestCard,
            ..RuleSet::default()
        };
        let left = classify(
            &[
                Card::suited(0, Spade, Rank::Four),
                Card::suited(0, Diamond, Rank::Four),
            ],
            &highest,
        )
        .unwrap();
        let right = classify(
            &[
                Card::suited(1, Spade, Rank::Four),
                Card::suited(0, Club, Rank::Four),
            ],
            &highest,
        )
        .unwrap();

        assert_eq!(
            compare_plays(&right, &left, &highest),
            PlayComparison::Equivalent
        );

        let lexicographic = RuleSet {
            suit_comparison: SuitComparison::Lexicographic,
            ..highest
        };
        assert_eq!(
            compare_plays(&right, &left, &lexicographic),
            PlayComparison::Greater
        );

        let sum_points = RuleSet {
            suit_comparison: SuitComparison::SumPoints,
            ..highest
        };
        assert_eq!(
            compare_plays(&right, &left, &sum_points),
            PlayComparison::Greater
        );

        let spade_diamond = classify(
            &[
                Card::suited(0, Spade, Rank::Six),
                Card::suited(0, Diamond, Rank::Six),
            ],
            &highest,
        )
        .unwrap();
        let heart_club = classify(
            &[
                Card::suited(0, Heart, Rank::Six),
                Card::suited(0, Club, Rank::Six),
            ],
            &highest,
        )
        .unwrap();
        assert_eq!(
            compare_plays(&spade_diamond, &heart_club, &lexicographic),
            PlayComparison::Greater
        );
        assert_eq!(
            compare_plays(&spade_diamond, &heart_club, &sum_points),
            PlayComparison::Equivalent
        );
    }

    #[test]
    fn jokers_use_spade_and_club_suit_points() {
        assert_eq!(Card::suited(0, Spade, Rank::Joker).suit_points(), 4);
        assert_eq!(Card::suited(0, Club, Rank::Joker).suit_points(), 2);
    }

    #[test]
    fn recognizes_airplanes_and_only_allows_matching_airplanes_to_follow() {
        let rules = RuleSet::default();
        let airplane = classify(
            &[
                Card::suited(0, Diamond, Rank::Six),
                Card::suited(0, Club, Rank::Six),
                Card::suited(0, Heart, Rank::Six),
                Card::suited(0, Diamond, Rank::Eight),
                Card::suited(0, Club, Rank::Eight),
                Card::suited(0, Heart, Rank::Eight),
            ],
            &rules,
        )
        .unwrap();
        let higher_airplane = classify(
            &[
                Card::suited(0, Diamond, Rank::Eight),
                Card::suited(0, Club, Rank::Eight),
                Card::suited(0, Heart, Rank::Eight),
                Card::suited(0, Diamond, Rank::Nine),
                Card::suited(0, Club, Rank::Nine),
                Card::suited(0, Heart, Rank::Nine),
            ],
            &rules,
        )
        .unwrap();
        let straight = classify(
            &[
                card(Rank::Four),
                card(Rank::Six),
                card(Rank::Eight),
                card(Rank::Nine),
                card(Rank::Ten),
                card(Rank::Jack),
            ],
            &rules,
        )
        .unwrap();

        assert_eq!(airplane.kind(), &PlayKind::Airplane { triple_count: 2 });
        assert_eq!(
            compare_plays(&higher_airplane, &airplane, &rules),
            PlayComparison::Greater
        );
        assert_eq!(
            compare_plays(&airplane, &straight, &rules),
            PlayComparison::Incompatible
        );
    }

    #[test]
    fn heaven_bomb_beats_every_other_shape_and_obeys_same_play_tie_rules() {
        let strict = RuleSet {
            deck_count: 2,
            ..RuleSet::default()
        };
        let heaven_cards = |deck, joker_suit| {
            [
                Card::suited(deck, Diamond, Rank::Three),
                Card::suited(deck, Diamond, Rank::Two),
                Card::suited(deck, Diamond, Rank::Five),
                Card::suited(deck, joker_suit, Rank::Joker),
                Card::suited(deck, Diamond, Rank::Seven),
            ]
        };
        let heaven = classify(&heaven_cards(0, Club), &strict).unwrap();
        let physical_copy = classify(&heaven_cards(1, Club), &strict).unwrap();
        let ordinary_bomb = classify(
            &[
                Card::suited(0, Diamond, Rank::Seven),
                Card::suited(0, Club, Rank::Seven),
                Card::suited(0, Heart, Rank::Seven),
                Card::suited(0, Spade, Rank::Seven),
            ],
            &strict,
        )
        .unwrap();

        assert_eq!(heaven.kind(), &PlayKind::HeavenBomb);
        assert_eq!(
            compare_plays(&heaven, &ordinary_bomb, &strict),
            PlayComparison::Greater
        );
        assert_eq!(
            compare_plays(&ordinary_bomb, &heaven, &strict),
            PlayComparison::Lower
        );
        assert_eq!(
            compare_plays(&physical_copy, &heaven, &strict),
            PlayComparison::Equivalent
        );
        assert!(!can_beat(&physical_copy, &heaven, &strict));
        let following = RuleSet {
            same_card_policy: SameCardPolicy::CanFollow,
            ..strict
        };
        assert!(can_beat(&physical_copy, &heaven, &following));

        let stronger_suit = classify(
            &[
                Card::suited(0, Diamond, Rank::Three),
                Card::suited(0, Diamond, Rank::Two),
                Card::suited(0, Diamond, Rank::Five),
                Card::suited(0, Spade, Rank::Joker),
                Card::suited(0, Spade, Rank::Seven),
            ],
            &strict,
        )
        .unwrap();
        assert_eq!(
            compare_plays(&stronger_suit, &heaven, &strict),
            PlayComparison::Greater
        );
    }
}
