use super::voices::ShengjiVoiceKind;
use leocard_shengji::{
    Category, ShengjiCard, ShengjiClassifiedPlay, ShengjiRank, ShengjiRuleSet, ShengjiSuit,
    ShengjiTrump, classify_lead,
};

fn play(suit: ShengjiSuit, ranks: &[(ShengjiRank, u8)]) -> ShengjiClassifiedPlay {
    let cards = ranks
        .iter()
        .flat_map(|&(rank, copies)| {
            (0..copies).map(move |deck| ShengjiCard::suited(deck, suit, rank))
        })
        .collect::<Vec<_>>();
    let trump = ShengjiTrump::new(ShengjiRank::Two, Some(ShengjiSuit::Heart)).unwrap();
    classify_lead(&cards, trump, &ShengjiRuleSet::default(), &[])
        .unwrap()
        .effective()
        .clone()
}

#[test]
fn trump_lead_voice_requires_a_leading_single_or_pair() {
    for copies in [1, 2] {
        let trump = play(ShengjiSuit::Heart, &[(ShengjiRank::Ace, copies)]);
        assert_eq!(
            ShengjiVoiceKind::for_play(&trump, true),
            Some(ShengjiVoiceKind::TrumpLead)
        );
        assert_eq!(ShengjiVoiceKind::for_play(&trump, false), None);
        let plain = play(ShengjiSuit::Spade, &[(ShengjiRank::Ace, copies)]);
        assert_eq!(ShengjiVoiceKind::for_play(&plain, true), None);
    }
    for (ranks, expected) in [
        (vec![(ShengjiRank::Ace, 3)], ShengjiVoiceKind::Triple),
        (vec![(ShengjiRank::Ace, 4)], ShengjiVoiceKind::Bomb),
        (
            vec![(ShengjiRank::King, 2), (ShengjiRank::Ace, 2)],
            ShengjiVoiceKind::Tractor,
        ),
        (
            vec![(ShengjiRank::King, 3), (ShengjiRank::Ace, 3)],
            ShengjiVoiceKind::Titanic,
        ),
        (
            vec![(ShengjiRank::King, 4), (ShengjiRank::Ace, 4)],
            ShengjiVoiceKind::Spaceship,
        ),
    ] {
        let trump = play(ShengjiSuit::Heart, &ranks);
        for is_lead in [true, false] {
            assert_eq!(ShengjiVoiceKind::for_play(&trump, is_lead), Some(expected));
        }
    }
}

#[test]
fn mixed_follows_do_not_announce_a_throw() {
    let mut cards = play(
        ShengjiSuit::Heart,
        &[(ShengjiRank::Queen, 1), (ShengjiRank::Ace, 1)],
    );
    assert_eq!(
        ShengjiVoiceKind::for_play(&cards, true),
        Some(ShengjiVoiceKind::Throw)
    );
    assert_eq!(ShengjiVoiceKind::for_play(&cards, false), None);
    cards.category = Category::Mixed;
    assert_eq!(ShengjiVoiceKind::for_play(&cards, false), None);
}
