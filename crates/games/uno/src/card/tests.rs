use super::*;

#[test]
fn classic_deck_has_108_unique_physical_cards() {
    let deck = build_deck();
    assert_eq!(deck.len(), 108);
    assert_eq!(
        deck.iter()
            .copied()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        108
    );
    assert_eq!(
        deck.iter()
            .filter(|card| card.face() == UnoFace::Wild)
            .count(),
        4
    );
    assert_eq!(
        deck.iter()
            .filter(|card| card.face() == UnoFace::WildDrawFour)
            .count(),
        4
    );
}

#[test]
fn swap_pack_adds_sixteen_unique_cards() {
    let cards = build_swap_pack();
    assert_eq!(cards.len(), 16);
    assert_eq!(
        cards
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        16
    );
    assert_eq!(
        build_deck_for_rules(crate::UnoRuleSet {
            swap_pack: true,
            ..crate::UnoRuleSet::default()
        })
        .len(),
        124
    );
    assert!(cards.iter().all(|card| card.face().is_swap_pack()));
    assert!(build_deck().iter().all(|card| !card.face().is_swap_pack()));
}

#[test]
fn reverse_pack_adds_sixteen_unique_cards() {
    let cards = build_reverse_pack();
    assert_eq!(cards.len(), 16);
    assert_eq!(
        cards
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        16
    );
    assert_eq!(
        build_deck_for_rules(crate::UnoRuleSet {
            reverse_pack: true,
            ..crate::UnoRuleSet::default()
        })
        .len(),
        124
    );
    assert!(cards.iter().all(|card| card.face().is_reverse_pack()));
    assert!(build_deck().iter().all(|card| !card.face().is_extension()));
}

#[test]
fn stack_pack_adds_sixteen_unique_cards() {
    let cards = build_stack_pack();
    assert_eq!(cards.len(), 16);
    assert_eq!(
        cards
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        16
    );
    assert_eq!(
        build_deck_for_rules(crate::UnoRuleSet {
            stack_pack: true,
            ..crate::UnoRuleSet::default()
        })
        .len(),
        124
    );
    assert!(cards.iter().all(|card| card.face().is_stack_pack()));
}

#[test]
fn no_mercy_deck_has_the_official_168_card_distribution() {
    let deck = build_no_mercy_deck();
    assert_eq!(deck.len(), 168);
    assert_eq!(
        deck.iter()
            .copied()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        168
    );
    for color in UnoColor::LIGHT {
        assert_eq!(
            deck.iter()
                .filter(|card| card.color() == Some(color))
                .count(),
            36
        );
        assert_eq!(
            deck.iter()
                .filter(|card| card.color() == Some(color) && card.face() == UnoFace::DrawTwo)
                .count(),
            3
        );
        assert_eq!(
            deck.iter()
                .filter(|card| card.color() == Some(color) && card.face() == UnoFace::DrawFour)
                .count(),
            2
        );
        assert_eq!(
            deck.iter()
                .filter(|card| card.color() == Some(color) && card.face() == UnoFace::SkipEveryone)
                .count(),
            2
        );
        assert_eq!(
            deck.iter()
                .filter(|card| card.color() == Some(color) && card.face() == UnoFace::DiscardAll)
                .count(),
            3
        );
    }
    assert_eq!(
        deck.iter()
            .filter(|card| card.face() == UnoFace::WildReverseDrawFour)
            .count(),
        8
    );
    assert_eq!(
        deck.iter()
            .filter(|card| card.face() == UnoFace::WildDrawSix)
            .count(),
        4
    );
    assert_eq!(
        deck.iter()
            .filter(|card| card.face() == UnoFace::WildDrawTen)
            .count(),
        4
    );
    assert_eq!(
        deck.iter()
            .filter(|card| card.face() == UnoFace::WildColorRoulette)
            .count(),
        8
    );
    assert!(deck.iter().all(|card| !card.face().is_extension()));
}

#[test]
fn flip_deck_has_complete_unique_light_and_dark_sides() {
    let deck = build_flip_deck();
    assert_eq!(deck.len(), 112);
    assert!(deck.iter().all(|card| card.is_double_sided()));
    assert_eq!(
        deck.iter()
            .copied()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        112
    );
    assert!(
        deck.iter()
            .all(|card| card.color().is_none_or(UnoColor::is_light))
    );
    assert!(deck.iter().all(|card| {
        card.opposite()
            .and_then(CardSide::color)
            .is_none_or(UnoColor::is_dark)
    }));
    assert_eq!(
        deck.iter()
            .filter(|card| card.face() == UnoFace::WildDrawTwo)
            .count(),
        4
    );
    assert_eq!(
        deck.iter()
            .filter(|card| card
                .opposite()
                .is_some_and(|side| { side.face() == UnoFace::WildDrawColor }))
            .count(),
        4
    );
}
