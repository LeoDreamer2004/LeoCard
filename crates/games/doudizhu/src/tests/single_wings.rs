use super::cards;
use crate::{
    DoudizhuAction, DoudizhuGameError, DoudizhuGameState, DoudizhuMode, DoudizhuPlayKind,
    DoudizhuRuleSet, build_deck, classify, hint_options,
};
use leocard_game_common::PlayerId;

#[test]
fn four_player_single_wings_are_optional_while_pairs_and_bare_triples_remain_legal() {
    let four = DoudizhuRuleSet::for_mode(DoudizhuMode::Four);
    assert!(!four.allow_single_wings);
    let enabled = DoudizhuRuleSet {
        allow_single_wings: true,
        ..four
    };
    for (ranks, kind) in [
        (vec![0, 0, 0, 4], DoudizhuPlayKind::TripleSingle),
        (
            vec![0, 0, 0, 1, 1, 1, 4, 5],
            DoudizhuPlayKind::AirplaneSingles,
        ),
        (
            vec![0, 0, 0, 1, 1, 1, 4, 4],
            DoudizhuPlayKind::AirplaneSingles,
        ),
    ] {
        let hand = cards(&ranks);
        assert!(classify(&hand, four).is_none());
        assert_eq!(classify(&hand, enabled).unwrap().kind, kind);
        assert_eq!(
            classify(&hand, DoudizhuRuleSet::default()).unwrap().kind,
            kind
        );
        assert!(hint_options(&hand, None, four).iter().all(|play| !matches!(
            play.kind,
            DoudizhuPlayKind::TripleSingle | DoudizhuPlayKind::AirplaneSingles
        )));
        assert!(
            hint_options(&hand, None, enabled)
                .iter()
                .any(|play| play.kind == kind)
        );
    }
    for (ranks, kind) in [
        (vec![0, 0, 0], DoudizhuPlayKind::Triple),
        (vec![0, 0, 0, 4, 4], DoudizhuPlayKind::TriplePair),
        (vec![0, 0, 0, 1, 1, 1], DoudizhuPlayKind::Airplane),
        (
            vec![0, 0, 0, 1, 1, 1, 4, 4, 5, 5],
            DoudizhuPlayKind::AirplanePairs,
        ),
        (vec![0, 0, 0, 0, 4, 5], DoudizhuPlayKind::FourSingles),
    ] {
        assert_eq!(classify(&cards(&ranks), four).unwrap().kind, kind);
    }
}

#[test]
fn authoritative_play_rejects_disabled_single_wings_without_consuming_cards() {
    for ranks in [vec![0, 0, 0, 4], vec![0, 0, 0, 1, 1, 1, 4, 5]] {
        let wanted = cards(&ranks);
        for enabled in [false, true] {
            let rules = DoudizhuRuleSet {
                allow_single_wings: enabled,
                ..DoudizhuRuleSet::for_mode(DoudizhuMode::Four)
            };
            let deck = build_deck(rules.mode);
            let mut remaining = deck.iter().copied().filter(|card| !wanted.contains(card));
            let arranged = (0..deck.len())
                .map(|index| {
                    if index % 4 == 0 && index / 4 < wanted.len() {
                        wanted[index / 4]
                    } else {
                        remaining.next().unwrap()
                    }
                })
                .collect();
            let mut game = DoudizhuGameState::new(rules, arranged, PlayerId(0)).unwrap();
            game.act(PlayerId(0), DoudizhuAction::Bid(3)).unwrap();
            let before = game.hand(PlayerId(0)).unwrap().to_vec();
            let result = game.act(PlayerId(0), DoudizhuAction::Play(wanted.clone()));
            if enabled {
                assert!(result.is_ok());
                assert_eq!(
                    game.hand(PlayerId(0)).unwrap().len(),
                    before.len() - wanted.len()
                );
            } else {
                assert_eq!(result, Err(DoudizhuGameError::InvalidPattern));
                assert_eq!(game.hand(PlayerId(0)).unwrap(), before);
                assert_eq!(game.current, PlayerId(0));
                assert!(game.table.is_none());
                assert!(game.played.is_empty());
            }
        }
    }
}
