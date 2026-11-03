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
