use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};
use std::fmt;

use crate::play::classify_cards;
use crate::{Card, ClassifiedPlay, Component, FollowError, PlayError, Rank, Trump};

/// 双升最小贪心机器人作出一次出牌决定所需的信息。
///
/// `lead` 为 `None` 时机器人领出最小单张；否则严格履行跟门、对子和拖拉机义务。
#[derive(Clone, Copy, Debug)]
pub struct GreedyBotRequest<'a> {
    pub hand: &'a [Card],
    pub lead: Option<&'a ClassifiedPlay>,
    pub trump: Trump,
}

/// 确定性的最小贪心策略，不保存对局状态，也不读取其他玩家手牌。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GreedyBot;

impl GreedyBot {
    pub fn choose(request: GreedyBotRequest<'_>) -> Result<ClassifiedPlay, GreedyBotError> {
        let Some(lead) = request.lead else {
            let card = request
                .hand
                .iter()
                .copied()
                .min_by(|left, right| card_cmp(*left, *right, request.trump))
                .ok_or(GreedyBotError::EmptyHand)?;
            return classify_cards(&[card], request.trump).map_err(GreedyBotError::InvalidLead);
        };

        let required = lead.cards.len();
        if request.hand.len() < required {
            return Err(GreedyBotError::NotEnoughCards {
                required,
                actual: request.hand.len(),
            });
        }
        if let Some(play) = crate::follow_suggestions(request.hand, lead, request.trump, 1)
            .into_iter()
            .next()
        {
            return Ok(play);
        }
        let lead_category = lead.category;
        let mut same_category = request
            .hand
            .iter()
            .copied()
            .filter(|card| crate::play::category(*card, request.trump) == lead_category)
            .collect::<Vec<_>>();
        same_category.sort_by(|left, right| card_cmp(*left, *right, request.trump));

        let cards = if same_category.len() < required {
            choose_short_category_fill(request.hand, &same_category, required, request.trump)
        } else {
            choose_structured_follow(&same_category, lead, request.trump)
        };
        crate::validate_follow(request.hand, &cards, lead, request.trump)
            .map_err(GreedyBotError::InvalidFollow)
    }
}

fn choose_short_category_fill(
    hand: &[Card],
    same_category: &[Card],
    required: usize,
    trump: Trump,
) -> Vec<Card> {
    let mut selected = same_category.to_vec();
    let selected_set = selected.iter().copied().collect::<HashSet<_>>();
    let mut fillers = hand
        .iter()
        .copied()
        .filter(|card| !selected_set.contains(card))
        .collect::<Vec<_>>();
    // 缺门时先垫任意最小副牌；副牌不足或只剩主牌时再从最小主牌开始补足。
    fillers.sort_by(|left, right| card_cmp(*left, *right, trump));
    selected.extend(fillers.into_iter().take(required - selected.len()));
    selected
}

fn choose_structured_follow(
    same_category: &[Card],
    lead: &ClassifiedPlay,
    trump: Trump,
) -> Vec<Card> {
    let required = lead.cards.len();
    let (tractor_demands, pair_slots) = structure_demands(lead);
    let all_pairs = pair_choices(same_category, trump);
    let mut obligation_pairs = all_pairs.clone();
    let mut required_runs = Vec::new();
    for maximum in tractor_demands {
        if let Some(run) = highest_run_at_most(&obligation_pairs, maximum) {
            if run.len() >= 2 {
                required_runs.push(run.len());
                remove_pair_indices(&mut obligation_pairs, &run);
            }
        }
    }
    let required_pair_count = pair_slots.min(all_pairs.len());

    let mut available_pairs = all_pairs;
    let mut selected = Vec::with_capacity(required);
    for run_len in required_runs {
        if let Some(run) = lowest_run_exact(&available_pairs, run_len) {
            for index in &run {
                selected.extend(available_pairs[*index].cards);
            }
            remove_pair_indices(&mut available_pairs, &run);
        }
    }
    while selected.len() / 2 < required_pair_count {
        let Some((index, pair)) = available_pairs
            .iter()
            .enumerate()
            .min_by(|(_, left), (_, right)| pair_cmp(left, right))
        else {
            break;
        };
        selected.extend(pair.cards);
        available_pairs.remove(index);
    }

    let selected_set = selected.iter().copied().collect::<HashSet<_>>();
    let mut singles = same_category
        .iter()
        .copied()
        .filter(|card| !selected_set.contains(card))
        .collect::<Vec<_>>();
    singles.sort_by(|left, right| card_cmp(*left, *right, trump));
    selected.extend(singles.into_iter().take(required - selected.len()));
    selected
}

fn structure_demands(lead: &ClassifiedPlay) -> (Vec<usize>, usize) {
    let mut tractors = Vec::new();
    let mut pair_slots = 0;
    for component in &lead.components {
        match component {
            Component::Single { .. } => {}
            Component::Pair { .. } => pair_slots += 1,
            Component::Triple { .. } => pair_slots += 1,
            Component::Quad { .. } => pair_slots += 2,
            Component::Tractor { pair_count, .. } => {
                let pair_count = usize::from(*pair_count);
                tractors.push(pair_count);
                pair_slots += pair_count;
            }
            Component::Titanic { triple_count, .. } => {
                let triple_count = usize::from(*triple_count);
                tractors.push(triple_count);
                pair_slots += triple_count;
            }
            Component::Spaceship { quad_count, .. } => {
                let pair_count = usize::from(*quad_count) * 2;
                tractors.push(pair_count);
                pair_slots += pair_count;
            }
        }
    }
    tractors.sort_unstable_by(|left, right| right.cmp(left));
    (tractors, pair_slots)
}

#[derive(Clone, Debug)]
struct PairChoice {
    cards: [Card; 2],
    strength: u8,
}

fn pair_choices(cards: &[Card], trump: Trump) -> Vec<PairChoice> {
    let mut faces: BTreeMap<(Option<crate::Suit>, Rank), Vec<Card>> = BTreeMap::new();
    for card in cards {
        faces
            .entry((card.suit(), card.rank()))
            .or_default()
            .push(*card);
    }
    faces
        .into_values()
        .filter_map(|mut cards| {
            if cards.len() < 2 {
                return None;
            }
            cards.sort_by_key(|card| card.deck());
            Some(PairChoice {
                cards: [cards[0], cards[1]],
                strength: trump.strength(cards[0]),
            })
        })
        .collect()
}

fn highest_run_at_most(pairs: &[PairChoice], maximum: usize) -> Option<Vec<usize>> {
    (2..=maximum)
        .rev()
        .find_map(|length| highest_run_exact(pairs, length))
}

fn highest_run_exact(pairs: &[PairChoice], length: usize) -> Option<Vec<usize>> {
    pairs
        .iter()
        .filter_map(|start| run_from(pairs, start.strength, length))
        .max_by_key(|run| pairs[*run.last().unwrap()].strength)
}

fn lowest_run_exact(pairs: &[PairChoice], length: usize) -> Option<Vec<usize>> {
    pairs
        .iter()
        .filter_map(|start| run_from(pairs, start.strength, length))
        .min_by_key(|run| pairs[*run.last().unwrap()].strength)
}

fn run_from(pairs: &[PairChoice], start: u8, length: usize) -> Option<Vec<usize>> {
    (0..length)
        .map(|offset| {
            pairs
                .iter()
                .position(|pair| pair.strength == start + offset as u8)
        })
        .collect()
}

fn remove_pair_indices(pairs: &mut Vec<PairChoice>, indices: &[usize]) {
    let mut indices = indices.to_vec();
    indices.sort_unstable();
    indices.dedup();
    for index in indices.into_iter().rev() {
        pairs.remove(index);
    }
}

fn pair_cmp(left: &PairChoice, right: &PairChoice) -> Ordering {
    left.strength.cmp(&right.strength).then_with(|| {
        left.cards[0]
            .suit()
            .cmp(&right.cards[0].suit())
            .then_with(|| left.cards[0].deck().cmp(&right.cards[0].deck()))
    })
}

fn card_cmp(left: Card, right: Card, trump: Trump) -> Ordering {
    card_key(left, trump).cmp(&card_key(right, trump))
}

fn card_key(card: Card, trump: Trump) -> (bool, u8, u8, u8) {
    (
        trump.is_trump(card),
        trump.strength(card),
        card.suit().map_or(4, crate::Suit::bid_strength),
        card.deck(),
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GreedyBotError {
    EmptyHand,
    NotEnoughCards { required: usize, actual: usize },
    InvalidLead(PlayError),
    InvalidFollow(FollowError),
}

impl fmt::Display for GreedyBotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyHand => f.write_str("机器人没有可出的牌"),
            Self::NotEnoughCards { required, actual } => {
                write!(f, "需要跟出 {required} 张牌，手中只有 {actual} 张")
            }
            Self::InvalidLead(error) => error.fmt(f),
            Self::InvalidFollow(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for GreedyBotError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Suit, classify_lead};

    fn card(deck: u8, suit: Suit, rank: Rank) -> Card {
        Card::suited(deck, suit, rank)
    }

    fn pair(suit: Suit, rank: Rank) -> [Card; 2] {
        [card(0, suit, rank), card(1, suit, rank)]
    }

    fn trump() -> Trump {
        Trump::new(Rank::Ten, Some(Suit::Heart)).unwrap()
    }

    fn lead(cards: &[Card]) -> ClassifiedPlay {
        let crate::TrickPlay::Accepted(play) =
            classify_lead(cards, trump(), &crate::RuleSet::default(), &[]).unwrap()
        else {
            unreachable!("test leads do not throw")
        };
        play
    }

    #[test]
    fn lead_uses_the_smallest_non_trump_single() {
        let hand = [
            card(0, Suit::Heart, Rank::Two),
            card(0, Suit::Diamond, Rank::Three),
            card(0, Suit::Club, Rank::Two),
        ];
        let play = GreedyBot::choose(GreedyBotRequest {
            hand: &hand,
            lead: None,
            trump: trump(),
        })
        .unwrap();

        assert_eq!(play.cards, vec![card(0, Suit::Club, Rank::Two)]);
    }

    #[test]
    fn following_a_pair_uses_the_smallest_available_pair() {
        let lead = lead(&pair(Suit::Spade, Rank::King));
        let low_single = card(0, Suit::Spade, Rank::Two);
        let low_pair = pair(Suit::Spade, Rank::Four);
        let hand = [
            &[low_single],
            low_pair.as_slice(),
            pair(Suit::Spade, Rank::Eight).as_slice(),
        ]
        .concat();
        let play = GreedyBot::choose(GreedyBotRequest {
            hand: &hand,
            lead: Some(&lead),
            trump: trump(),
        })
        .unwrap();

        assert_eq!(play.cards, low_pair);
    }

    #[test]
    fn following_a_tractor_uses_the_smallest_available_tractor() {
        let lead = lead(
            &[
                pair(Suit::Spade, Rank::King),
                pair(Suit::Spade, Rank::Queen),
            ]
            .concat(),
        );
        let low = [
            pair(Suit::Spade, Rank::Three),
            pair(Suit::Spade, Rank::Four),
        ]
        .concat();
        let high = [
            pair(Suit::Spade, Rank::Eight),
            pair(Suit::Spade, Rank::Seven),
        ]
        .concat();
        let hand = [low.clone(), high].concat();
        let play = GreedyBot::choose(GreedyBotRequest {
            hand: &hand,
            lead: Some(&lead),
            trump: trump(),
        })
        .unwrap();

        assert_eq!(play.cards, low);
    }

    #[test]
    fn void_discard_uses_smallest_off_suits_before_any_trump() {
        let lead = lead(&[
            card(0, Suit::Spade, Rank::Five),
            card(0, Suit::Spade, Rank::Six),
            card(0, Suit::Spade, Rank::Seven),
        ]);
        let expected = [
            card(0, Suit::Diamond, Rank::Two),
            card(0, Suit::Club, Rank::Three),
            card(0, Suit::Diamond, Rank::Four),
        ];
        let hand = [
            expected[2],
            card(0, Suit::Heart, Rank::Two),
            expected[1],
            Card::small_joker(0),
            expected[0],
        ];
        let play = GreedyBot::choose(GreedyBotRequest {
            hand: &hand,
            lead: Some(&lead),
            trump: trump(),
        })
        .unwrap();

        assert_eq!(play.category, crate::Category::Mixed);
        assert_eq!(play.cards, expected);
    }

    #[test]
    fn void_hand_with_only_trumps_uses_the_smallest_trumps() {
        let lead = lead(&[
            card(0, Suit::Spade, Rank::Five),
            card(0, Suit::Spade, Rank::Six),
        ]);
        let expected = [
            card(0, Suit::Heart, Rank::Two),
            card(0, Suit::Heart, Rank::Three),
        ];
        let hand = [
            Card::small_joker(0),
            expected[1],
            card(0, Suit::Spade, Rank::Ten),
            expected[0],
        ];
        let play = GreedyBot::choose(GreedyBotRequest {
            hand: &hand,
            lead: Some(&lead),
            trump: trump(),
        })
        .unwrap();

        assert_eq!(play.cards, expected);
    }
}
