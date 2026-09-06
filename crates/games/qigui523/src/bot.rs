use std::cmp::Ordering;
use std::collections::HashSet;

use crate::{
    ClassifiedPlay, PlayComparison, QiGuiCard, QiGuiRuleSet, can_beat, classify, compare_plays,
};

/// 一次贪心跟牌请求所需的公开信息。
///
/// `played_cards` 应包含调用方目前已经观察到的公开出牌。它既不会泄露暗牌，
/// 也不会直接改变合法性；当公开历史发生变化时，它会让有状态策略开始一轮新的搜索。
#[derive(Clone, Copy, Debug)]
pub struct QiGui523BotRequest<'a> {
    pub hand: &'a [QiGuiCard],
    pub current_play: &'a ClassifiedPlay,
    pub played_cards: &'a [QiGuiCard],
    pub rules: &'a QiGuiRuleSet,
}

/// 可重复请求的贪心跟牌机器人。
///
/// 第一次请求返回最小的合法跟牌，保持请求上下文不变再次调用则依次返回次小候选。
/// 手牌、当前桌面牌、公开出牌历史或规则变化后，候选游标会自动重置。
#[derive(Clone, Debug, Default)]
pub struct QiGui523Bot {
    search: Option<GreedySearch>,
}

/// 判断当前手牌是否存在至少一种能够压过桌面牌的合法选择。
///
/// 该查询不保存或推进 [`QiGui523Bot`] 的候选游标，适合 UI 在显示按钮前探测。
pub fn has_legal_response(request: QiGui523BotRequest<'_>) -> bool {
    !legal_responses(request).is_empty()
}

impl QiGui523Bot {
    pub fn new() -> Self {
        Self::default()
    }

    /// 返回当前上下文中的下一个贪心候选；耗尽时返回 `None`。
    pub fn choose(&mut self, request: QiGui523BotRequest<'_>) -> Option<ClassifiedPlay> {
        let key = GreedyContextKey::from_request(request);
        if self.search.as_ref().is_none_or(|search| search.key != key) {
            self.search = Some(GreedySearch {
                candidates: legal_responses(request),
                key,
                next_index: 0,
            });
        }

        let search = self.search.as_mut().expect("search was initialized above");
        let candidate = search.candidates.get(search.next_index)?.clone();
        search.next_index += 1;
        Some(candidate)
    }

    /// 丢弃当前搜索状态，使下次请求重新从最小候选开始。
    pub fn reset(&mut self) {
        self.search = None;
    }

    /// 当前上下文还没有返回的候选数量。
    pub fn remaining(&self) -> usize {
        self.search.as_ref().map_or(0, |search| {
            search.candidates.len().saturating_sub(search.next_index)
        })
    }
}

#[derive(Clone, Debug)]
struct GreedySearch {
    key: GreedyContextKey,
    candidates: Vec<ClassifiedPlay>,
    next_index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GreedyContextKey {
    hand: Vec<QiGuiCard>,
    current_play: Vec<QiGuiCard>,
    played_cards: Vec<QiGuiCard>,
    rules: QiGuiRuleSet,
}

impl GreedyContextKey {
    fn from_request(request: QiGui523BotRequest<'_>) -> Self {
        Self {
            hand: sorted_physical_cards(request.hand),
            current_play: sorted_physical_cards(request.current_play.cards()),
            played_cards: sorted_physical_cards(request.played_cards),
            rules: *request.rules,
        }
    }
}

fn legal_responses(request: QiGui523BotRequest<'_>) -> Vec<ClassifiedPlay> {
    let mut candidates = Vec::new();
    let mut semantic_choices = HashSet::new();
    let hand_len = request.hand.len();

    // 规则目前把手牌限制在十五张以内，子集穷举至多检查 32767 种组合。
    // 使用 checked_shl 可让模块在被其他调用方误用超大手牌时安全地返回空集。
    let Some(subset_count) = 1_usize.checked_shl(hand_len as u32) else {
        return candidates;
    };
    for mask in 1..subset_count {
        let cards = request
            .hand
            .iter()
            .enumerate()
            .filter_map(|(index, card)| ((mask >> index) & 1 == 1).then_some(*card))
            .collect::<Vec<_>>();
        let Ok(play) = classify(&cards, request.rules) else {
            continue;
        };
        if !can_beat(&play, request.current_play, request.rules) {
            continue;
        }

        // 多副牌中，仅物理 deck 编号不同的选择没有策略差异，不重复返回。
        if semantic_choices.insert(semantic_choice_key(&play)) {
            candidates.push(play);
        }
    }

    candidates.sort_by(|left, right| greedy_play_cmp(left, right, request.rules));
    candidates
}

fn greedy_play_cmp(
    left: &ClassifiedPlay,
    right: &ClassifiedPlay,
    rules: &QiGuiRuleSet,
) -> Ordering {
    match compare_plays(left, right, rules) {
        PlayComparison::Lower => Ordering::Less,
        PlayComparison::Greater => Ordering::Greater,
        PlayComparison::Equivalent | PlayComparison::Incompatible => {
            physical_cards_cmp(left.cards(), right.cards())
        }
    }
}

fn semantic_choice_key(play: &ClassifiedPlay) -> Vec<(u8, u8)> {
    let mut key = play
        .cards()
        .iter()
        .map(|card| (card.rank().strength(), card.suit().strength()))
        .collect::<Vec<_>>();
    key.sort_unstable();
    key
}

fn sorted_physical_cards(cards: &[QiGuiCard]) -> Vec<QiGuiCard> {
    let mut cards = cards.to_vec();
    cards.sort_by(card_physical_cmp);
    cards
}

fn physical_cards_cmp(left: &[QiGuiCard], right: &[QiGuiCard]) -> Ordering {
    left.len().cmp(&right.len()).then_with(|| {
        left.iter()
            .zip(right)
            .map(|(left, right)| card_physical_cmp(left, right))
            .find(|ordering| *ordering != Ordering::Equal)
            .unwrap_or(Ordering::Equal)
    })
}

fn card_physical_cmp(left: &QiGuiCard, right: &QiGuiCard) -> Ordering {
    left.rank()
        .strength()
        .cmp(&right.rank().strength())
        .then_with(|| left.suit().strength().cmp(&right.suit().strength()))
        .then_with(|| left.deck().cmp(&right.deck()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{QiGuiRank, QiGuiSuit, SameCardPolicy};

    fn card(deck: u8, suit: QiGuiSuit, rank: QiGuiRank) -> QiGuiCard {
        QiGuiCard::suited(deck, suit, rank)
    }

    fn single(card: QiGuiCard, rules: &QiGuiRuleSet) -> ClassifiedPlay {
        classify(&[card], rules).unwrap()
    }

    #[test]
    fn repeated_requests_walk_candidates_from_smallest_to_largest() {
        let rules = QiGuiRuleSet::default();
        let current = single(card(0, QiGuiSuit::Diamond, QiGuiRank::Eight), &rules);
        let hand = [
            card(0, QiGuiSuit::Diamond, QiGuiRank::Jack),
            card(0, QiGuiSuit::Diamond, QiGuiRank::Nine),
            card(0, QiGuiSuit::Diamond, QiGuiRank::Ten),
        ];
        let history = current.cards().to_vec();
        let request = QiGui523BotRequest {
            hand: &hand,
            current_play: &current,
            played_cards: &history,
            rules: &rules,
        };
        let mut strategy = QiGui523Bot::new();

        assert_eq!(
            strategy.choose(request).unwrap().cards(),
            &[card(0, QiGuiSuit::Diamond, QiGuiRank::Nine)]
        );
        assert_eq!(
            strategy.choose(request).unwrap().cards(),
            &[card(0, QiGuiSuit::Diamond, QiGuiRank::Ten)]
        );
        assert_eq!(
            strategy.choose(request).unwrap().cards(),
            &[card(0, QiGuiSuit::Diamond, QiGuiRank::Jack)]
        );
        assert_eq!(strategy.choose(request), None);
    }

    #[test]
    fn ordinary_responses_are_used_before_bombs() {
        let rules = QiGuiRuleSet::default();
        let current = single(card(0, QiGuiSuit::Diamond, QiGuiRank::Eight), &rules);
        let hand = [
            card(0, QiGuiSuit::Diamond, QiGuiRank::Nine),
            card(0, QiGuiSuit::Diamond, QiGuiRank::Six),
            card(0, QiGuiSuit::Club, QiGuiRank::Six),
            card(0, QiGuiSuit::Heart, QiGuiRank::Six),
            card(0, QiGuiSuit::Spade, QiGuiRank::Six),
        ];
        let mut strategy = QiGui523Bot::new();
        let request = QiGui523BotRequest {
            hand: &hand,
            current_play: &current,
            played_cards: current.cards(),
            rules: &rules,
        };

        let first = strategy.choose(request).unwrap();
        assert_eq!(
            first.cards(),
            &[card(0, QiGuiSuit::Diamond, QiGuiRank::Nine)]
        );
        let second = strategy.choose(request).unwrap();
        assert!(matches!(second.kind(), crate::QiGuiPlayKind::Bomb(_)));
    }

    #[test]
    fn greedy_search_finds_an_advanced_type_response() {
        let rules = QiGuiRuleSet {
            advanced_play_types: true,
            ..QiGuiRuleSet::default()
        };
        let current = classify(
            &[
                card(0, QiGuiSuit::Diamond, QiGuiRank::Eight),
                card(0, QiGuiSuit::Diamond, QiGuiRank::Nine),
                card(0, QiGuiSuit::Diamond, QiGuiRank::Ten),
            ],
            &rules,
        )
        .unwrap();
        let hand = [
            card(0, QiGuiSuit::Diamond, QiGuiRank::Four),
            card(0, QiGuiSuit::Club, QiGuiRank::Four),
            card(0, QiGuiSuit::Heart, QiGuiRank::Four),
        ];
        let request = QiGui523BotRequest {
            hand: &hand,
            current_play: &current,
            played_cards: current.cards(),
            rules: &rules,
        };
        let mut strategy = QiGui523Bot::new();

        assert!(has_legal_response(request));
        assert!(matches!(
            strategy.choose(request).unwrap().kind(),
            crate::QiGuiPlayKind::Triple
        ));
    }

    #[test]
    fn greedy_search_finds_a_higher_triple_with_single() {
        let rules = QiGuiRuleSet {
            advanced_play_types: true,
            ..QiGuiRuleSet::default()
        };
        let current = classify(
            &[
                card(0, QiGuiSuit::Diamond, QiGuiRank::Four),
                card(0, QiGuiSuit::Club, QiGuiRank::Four),
                card(0, QiGuiSuit::Heart, QiGuiRank::Four),
                card(0, QiGuiSuit::Diamond, QiGuiRank::Seven),
            ],
            &rules,
        )
        .unwrap();
        let hand = [
            card(0, QiGuiSuit::Diamond, QiGuiRank::Six),
            card(0, QiGuiSuit::Club, QiGuiRank::Six),
            card(0, QiGuiSuit::Heart, QiGuiRank::Six),
            card(0, QiGuiSuit::Diamond, QiGuiRank::Four),
        ];
        let request = QiGui523BotRequest {
            hand: &hand,
            current_play: &current,
            played_cards: current.cards(),
            rules: &rules,
        };
        let mut strategy = QiGui523Bot::new();

        assert!(has_legal_response(request));
        assert!(matches!(
            strategy.choose(request).unwrap().kind(),
            crate::QiGuiPlayKind::TripleWithSingle
        ));
    }

    #[test]
    fn changed_public_history_resets_the_saved_cursor() {
        let rules = QiGuiRuleSet::default();
        let current = single(card(0, QiGuiSuit::Diamond, QiGuiRank::Eight), &rules);
        let hand = [
            card(0, QiGuiSuit::Diamond, QiGuiRank::Nine),
            card(0, QiGuiSuit::Diamond, QiGuiRank::Ten),
        ];
        let initial_history = current.cards().to_vec();
        let later_history = [
            current.cards()[0],
            card(0, QiGuiSuit::Club, QiGuiRank::Four),
        ];
        let mut strategy = QiGui523Bot::new();

        let first_request = QiGui523BotRequest {
            hand: &hand,
            current_play: &current,
            played_cards: &initial_history,
            rules: &rules,
        };
        assert_eq!(strategy.choose(first_request).unwrap().cards(), &[hand[0]]);
        assert_eq!(strategy.choose(first_request).unwrap().cards(), &[hand[1]]);

        let changed_request = QiGui523BotRequest {
            played_cards: &later_history,
            ..first_request
        };
        assert_eq!(
            strategy.choose(changed_request).unwrap().cards(),
            &[hand[0]]
        );
    }

    #[test]
    fn equivalent_physical_copies_are_not_returned_twice() {
        let rules = QiGuiRuleSet {
            deck_count: 2,
            ..QiGuiRuleSet::default()
        };
        let current = single(card(0, QiGuiSuit::Diamond, QiGuiRank::Eight), &rules);
        let hand = [
            card(0, QiGuiSuit::Diamond, QiGuiRank::Nine),
            card(1, QiGuiSuit::Diamond, QiGuiRank::Nine),
            card(0, QiGuiSuit::Diamond, QiGuiRank::Ten),
        ];
        let request = QiGui523BotRequest {
            hand: &hand,
            current_play: &current,
            played_cards: current.cards(),
            rules: &rules,
        };
        let mut strategy = QiGui523Bot::new();

        assert_eq!(
            strategy.choose(request).unwrap().cards()[0].rank(),
            QiGuiRank::Nine
        );
        assert_eq!(
            strategy.choose(request).unwrap().cards()[0].rank(),
            QiGuiRank::Ten
        );
        assert_eq!(strategy.choose(request), None);
    }

    #[test]
    fn equal_strength_following_obeys_room_rules() {
        let current_card = card(0, QiGuiSuit::Diamond, QiGuiRank::Nine);
        let hand = [
            card(1, QiGuiSuit::Diamond, QiGuiRank::Nine),
            card(0, QiGuiSuit::Club, QiGuiRank::Nine),
        ];
        let strict = QiGuiRuleSet {
            deck_count: 2,
            same_card_policy: SameCardPolicy::MustBeHigher,
            ..QiGuiRuleSet::default()
        };
        let following = QiGuiRuleSet {
            same_card_policy: SameCardPolicy::CanFollow,
            ..strict
        };
        let current = single(current_card, &strict);

        let mut strict_strategy = QiGui523Bot::new();
        assert_eq!(
            strict_strategy
                .choose(QiGui523BotRequest {
                    hand: &hand,
                    current_play: &current,
                    played_cards: &[current_card],
                    rules: &strict,
                })
                .unwrap()
                .cards(),
            &[hand[1]]
        );

        let mut following_strategy = QiGui523Bot::new();
        assert_eq!(
            following_strategy
                .choose(QiGui523BotRequest {
                    hand: &hand,
                    current_play: &current,
                    played_cards: &[current_card],
                    rules: &following,
                })
                .unwrap()
                .cards(),
            &[hand[0]]
        );
    }
}
