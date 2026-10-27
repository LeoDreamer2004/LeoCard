use super::cards;
use crate::{
    Action, GameError, GameState, Mode, PlayKind, RuleSet, build_deck, classify, hint_options,
};

#[test]
fn four_player_single_wings_are_optional_while_pairs_and_bare_triples_remain_legal() {
    let four = RuleSet::for_mode(Mode::Four);
    assert!(!four.allow_single_wings);
    let enabled = RuleSet {
        allow_single_wings: true,
        ..four
    };
    for (ranks, kind) in [
        (vec![0, 0, 0, 4], PlayKind::TripleSingle),
        (vec![0, 0, 0, 1, 1, 1, 4, 5], PlayKind::AirplaneSingles),
        (vec![0, 0, 0, 1, 1, 1, 4, 4], PlayKind::AirplaneSingles),
    ] {
        let hand = cards(&ranks);
        assert!(classify(&hand, four).is_none());
        assert_eq!(classify(&hand, enabled).unwrap().kind, kind);
        assert_eq!(classify(&hand, RuleSet::default()).unwrap().kind, kind);
        assert!(hint_options(&hand, None, four).iter().all(|play| !matches!(
            play.kind,
            PlayKind::TripleSingle | PlayKind::AirplaneSingles
        )));
        assert!(
            hint_options(&hand, None, enabled)
                .iter()
                .any(|play| play.kind == kind)
        );
    }
    for (ranks, kind) in [
        (vec![0, 0, 0], PlayKind::Triple),
        (vec![0, 0, 0, 4, 4], PlayKind::TriplePair),
        (vec![0, 0, 0, 1, 1, 1], PlayKind::Airplane),
        (vec![0, 0, 0, 1, 1, 1, 4, 4, 5, 5], PlayKind::AirplanePairs),
        (vec![0, 0, 0, 0, 4, 5], PlayKind::FourSingles),
    ] {
        assert_eq!(classify(&cards(&ranks), four).unwrap().kind, kind);
    }
}

#[test]
fn authoritative_play_rejects_disabled_single_wings_without_consuming_cards() {
    for ranks in [vec![0, 0, 0, 4], vec![0, 0, 0, 1, 1, 1, 4, 5]] {
        let wanted = cards(&ranks);
        for enabled in [false, true] {
            let rules = RuleSet {
                allow_single_wings: enabled,
                ..RuleSet::for_mode(Mode::Four)
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
            let mut game = GameState::new(rules, arranged, 0).unwrap();
            game.act(0, Action::Bid(3)).unwrap();
            let before = game.hand(0).unwrap().to_vec();
            let result = game.act(0, Action::Play(wanted.clone()));
            if enabled {
                assert!(result.is_ok());
                assert_eq!(game.hand(0).unwrap().len(), before.len() - wanted.len());
            } else {
                assert_eq!(result, Err(GameError::InvalidPattern));
                assert_eq!(game.hand(0).unwrap(), before);
                assert_eq!(game.current, 0);
                assert!(game.table.is_none());
                assert!(game.played.is_empty());
            }
        }
    }
}
