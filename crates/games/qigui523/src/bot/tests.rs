use super::*;
use crate::{
    QiGuiCard, QiGuiClassifiedPlay, QiGuiRank, QiGuiRuleSet, QiGuiSameCardPolicy, QiGuiSuit,
    classify,
};

fn card(deck: u8, suit: QiGuiSuit, rank: QiGuiRank) -> QiGuiCard {
    QiGuiCard::suited(deck, suit, rank)
}

fn single(card: QiGuiCard, rules: &QiGuiRuleSet) -> QiGuiClassifiedPlay {
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
        same_card_policy: QiGuiSameCardPolicy::MustBeHigher,
        ..QiGuiRuleSet::default()
    };
    let following = QiGuiRuleSet {
        same_card_policy: QiGuiSameCardPolicy::CanFollow,
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
