use super::support::*;
use leocard_protocol::ShengjiDeclarationView;
use leocard_shengji::{
    Category, ShengjiBidKind, ShengjiBidTrump, ShengjiRedealReason, ShengjiRuleSet,
    ShengjiSuit as Suit, ShengjiTrump, TrickPlay, build_deck_for, classify_lead,
};

#[test]
fn bids_and_copy_award_the_actor_only() {
    for kind in [
        ShengjiBidKind::Initial,
        ShengjiBidKind::Protect,
        ShengjiBidKind::Counter,
        ShengjiBidKind::SelfCounter,
    ] {
        let declaration = ShengjiDeclarationView {
            player: PlayerId(1),
            trump: ShengjiBidTrump::NoTrumpBigJoker,
            kind,
            protected: false,
            cards: vec![Card::big_joker(0), Card::big_joker(1)],
        };
        let trigger = AchievementTrigger::Game {
            player: PlayerId(1),
            event: GameEvent::Shengji(ShengjiEvent::DeclarationChanged {
                declaration: declaration.clone(),
            }),
        };
        assert_eq!(
            amount("counter", &trigger),
            u64::from(matches!(
                kind,
                ShengjiBidKind::Counter | ShengjiBidKind::SelfCounter
            ))
        );
        assert_eq!(amount("no_trump", &trigger), 1);
        assert_eq!(amount("copy_bottom", &trigger), 0);
        let copy = GameEvent::Shengji(ShengjiEvent::BottomCopied { declaration });
        assert_eq!(
            amount(
                "copy_bottom",
                &AchievementTrigger::Game {
                    player: PlayerId(1),
                    event: copy.clone()
                }
            ),
            1
        );
        for id in ["counter", "no_trump", "copy_bottom"] {
            assert_eq!(
                amount(
                    id,
                    &AchievementTrigger::Game {
                        player: PlayerId(0),
                        event: copy.clone()
                    }
                ),
                0
            );
        }
    }
    let declaration = ShengjiDeclarationView {
        player: PlayerId(1),
        trump: ShengjiBidTrump::Suit(Suit::Heart),
        kind: ShengjiBidKind::Counter,
        protected: false,
        cards: vec![],
    };
    assert_eq!(
        amount(
            "no_trump",
            &AchievementTrigger::Game {
                player: PlayerId(1),
                event: ShengjiEvent::DeclarationChanged { declaration }.into()
            }
        ),
        0
    );
}

fn throwing(count: usize) -> ShengjiClassifiedPlay {
    let cards = build_deck_for(2)
        .into_iter()
        .filter(|card| card.suit() == Some(Suit::Spade) && card.rank() != Rank::Two)
        .take(count)
        .collect::<Vec<_>>();
    let trump = ShengjiTrump::new(Rank::Two, Some(Suit::Heart)).unwrap();
    let TrickPlay::Accepted(play) =
        classify_lead(&cards, trump, &ShengjiRuleSet::default(), &[]).unwrap()
    else {
        panic!("unopposed throw");
    };
    play
}

#[test]
fn throws_use_strict_lengths_and_only_successful_leads() {
    for (id, boundary) in [
        ("throw_eight", 8),
        ("throw_twelve", 12),
        ("throw_twenty", 20),
    ] {
        assert_eq!(amount(id, &played(throwing(boundary), true)), 0);
        assert_eq!(amount(id, &played(throwing(boundary + 1), true)), 1);
        assert_eq!(amount(id, &played(throwing(boundary + 1), false)), 0);
        let mut forced = throwing(1);
        forced.cards = throwing(boundary + 1).cards;
        assert_eq!(amount(id, &played(forced, true)), 0);
    }
}

#[test]
fn sequences_require_the_actual_component() {
    let card = Card::small_joker(0);
    for pairs in [3, 4] {
        let play = ShengjiClassifiedPlay {
            cards: vec![card; usize::from(pairs) * 2],
            category: Category::Trump,
            components: vec![Component::Tractor {
                cards: vec![card; usize::from(pairs) * 2],
                pair_count: pairs,
                top_strength: 9,
            }],
        };
        assert_eq!(
            amount("long_tractor", &played(play, false)),
            u64::from(pairs == 4)
        );
    }
    let cards = [Rank::Three, Rank::Four]
        .into_iter()
        .flat_map(|rank| (0..4).map(move |deck| Card::suited(deck, Suit::Heart, rank)))
        .collect::<Vec<_>>();
    let trump = ShengjiTrump::new(Rank::Two, Some(Suit::Spade)).unwrap();
    let rules = ShengjiRuleSet {
        deck_count: 4,
        ..ShengjiRuleSet::default()
    };
    let TrickPlay::Accepted(play) = classify_lead(&cards, trump, &rules, &[]).unwrap() else {
        panic!("spaceship");
    };
    assert_eq!(amount("spaceship", &played(play, true)), 1);
    assert_eq!(amount("spaceship", &played(throwing(9), true)), 0);
}

#[test]
fn redeal_requires_both_power_outage_and_exhausted_bottom() {
    for (reason, expected) in [
        (ShengjiRedealReason::NoDeclaration, 0),
        (
            ShengjiRedealReason::BottomFlipExhausted {
                power_outage_used: false,
            },
            0,
        ),
        (
            ShengjiRedealReason::BottomFlipExhausted {
                power_outage_used: true,
            },
            1,
        ),
    ] {
        let trigger = AchievementTrigger::Game {
            player: PlayerId(1),
            event: ShengjiEvent::RedealRequired { reason }.into(),
        };
        assert_eq!(amount("exhausted_redeal", &trigger), expected);
    }
}
