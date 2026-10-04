use super::support::*;
use crate::{UnoFace, build_deck, build_deck_for_rules};

#[test]
fn full_stack_tracks_all_twelve_physical_cards_and_the_original_sender() {
    let rules = UnoRuleSet {
        action_stacking: true,
        ..UnoRuleSet::default()
    };
    let mut stack = build_deck()
        .into_iter()
        .filter(|card| card.face() == UnoFace::DrawTwo)
        .collect::<Vec<_>>();
    stack.extend((0..4).map(|copy| UnoCard::wild(UnoFace::WildDrawFour, copy)));
    let mut hands = vec![Vec::new(), Vec::new()];
    for (index, card) in stack.iter().copied().enumerate() {
        hands[index % 2].push(card);
    }
    for hand in &mut hands {
        hand.push(number(UnoColor::Red, 9));
    }
    let (mut game, mut stats) = table(rules, hands, number(UnoColor::Red, 1));
    for (index, card) in stack.iter().copied().enumerate() {
        let color = card.face().is_wild().then_some(UnoColor::Red);
        record(
            &mut game,
            &mut stats,
            index % 2,
            &[(card, color)],
            false,
            |game, player| game.play_card(player, card, color),
        );
    }
    assert_eq!(game.turn().unwrap().pending_draw, 32);
    record(
        &mut game,
        &mut stats,
        0,
        &[],
        false,
        GameState::accept_draw_penalty,
    );
    assert!(progress(&stats, 0).full_draw_chain);
    assert!(progress(&stats, 0).draw_chain_returned);
    assert!(progress(&stats, 0).has_penalty);
    assert_eq!(progress(&stats, 0).max_penalty_cards, 32);
    assert!(!progress(&stats, 1).full_draw_chain);
}

#[test]
fn a_large_penalty_without_the_full_chain_does_not_unlock_the_grand_slam() {
    let (mut game, mut stats) = table(
        UnoRuleSet::default(),
        vec![
            vec![number(UnoColor::Blue, 1)],
            vec![number(UnoColor::Green, 2)],
        ],
        number(UnoColor::Red, 9),
    );
    game.pending_kind = Some(UnoPendingDrawKind::DrawTwo);
    game.pending_draw = 32;
    record(
        &mut game,
        &mut stats,
        0,
        &[],
        false,
        GameState::accept_draw_penalty,
    );
    assert!(!progress(&stats, 0).full_draw_chain);
    assert!(!progress(&stats, 0).draw_chain_returned);
}

#[test]
fn a_normal_draw_from_five_cards_can_eliminate_but_is_not_a_penalty() {
    let rules = UnoRuleSet {
        mode: Mode::NoMercy,
        ..UnoRuleSet::default()
    };
    let (mut game, mut stats) = table(
        rules,
        vec![
            vec![number(UnoColor::Blue, 1); 5],
            vec![number(UnoColor::Yellow, 2)],
        ],
        number(UnoColor::Red, 9),
    );
    game.draw_pile = build_deck_for_rules(rules)
        .into_iter()
        .filter(|card| {
            !card.face().is_wild()
                && card.color() != Some(UnoColor::Red)
                && card.face() != UnoFace::Number(9)
        })
        .collect();
    let facts = record(&mut game, &mut stats, 0, &[], false, GameState::draw_card);
    assert!(game.player(UnoPlayerId(0)).unwrap().eliminated());
    assert!(progress(&stats, 0).lucky_elimination);
    assert!(progress(&stats, 0).has_drawn);
    assert!(!progress(&stats, 0).has_penalty);
    assert_eq!(progress(&stats, 0).peak_hand, 25);
    assert!(!progress(&stats, 0).reached_twenty_four);
    assert!(
        facts
            .iter()
            .find(|facts| facts.player == UnoPlayerId(1))
            .unwrap()
            .all_opponents_eliminated
    );
}

#[test]
fn twenty_four_is_a_surviving_hand_size_and_skip_batches_are_separate_from_draws() {
    let rules = UnoRuleSet {
        mode: Mode::NoMercy,
        no_mercy: crate::NoMercyRuleSet {
            draw_until_playable: false,
            ..Default::default()
        },
        ..UnoRuleSet::default()
    };
    let (mut game, mut stats) = table(
        rules,
        vec![
            vec![number(UnoColor::Blue, 1); 23],
            vec![number(UnoColor::Yellow, 2)],
        ],
        number(UnoColor::Red, 9),
    );
    game.draw_pile = vec![number(UnoColor::Green, 1)].into();
    record(&mut game, &mut stats, 0, &[], false, GameState::draw_card);
    assert!(progress(&stats, 0).reached_twenty_four);
    assert!(!game.player(UnoPlayerId(0)).unwrap().eliminated());
    let rules = UnoRuleSet {
        action_stacking: true,
        ..UnoRuleSet::default()
    };
    let (mut game, mut stats) = table(
        rules,
        vec![
            vec![number(UnoColor::Blue, 1)],
            vec![number(UnoColor::Green, 2)],
        ],
        number(UnoColor::Red, 9),
    );
    game.pending_skip = 3;
    record(
        &mut game,
        &mut stats,
        0,
        &[],
        false,
        GameState::resolve_skip,
    );
    assert_eq!(progress(&stats, 0).max_skip_batch, 3);
    assert!(!progress(&stats, 0).has_drawn);
}

#[test]
fn uno_report_and_challenge_record_the_actual_recipient_and_result() {
    let wild = UnoCard::wild(UnoFace::WildDrawFour, 0);
    for legal in [false, true] {
        let spare = number(if legal { UnoColor::Blue } else { UnoColor::Red }, 1);
        let (mut game, mut stats) = table(
            UnoRuleSet::default(),
            vec![vec![wild, spare], vec![number(UnoColor::Green, 2)]],
            number(UnoColor::Red, 9),
        );
        record(
            &mut game,
            &mut stats,
            0,
            &[(wild, Some(UnoColor::Blue))],
            false,
            |game, player| game.play_card(player, wild, Some(UnoColor::Blue)),
        );
        record(
            &mut game,
            &mut stats,
            1,
            &[],
            false,
            GameState::challenge_draw_four,
        );
        assert_eq!(progress(&stats, 1).challenges, 1);
        assert_eq!(progress(&stats, 1).successful_challenges, u32::from(!legal));
        assert_eq!(progress(&stats, 0).challenges_received, 1);
        assert!(progress(&stats, usize::from(legal)).has_penalty);
        assert_eq!(
            progress(&stats, usize::from(legal)).max_penalty_cards,
            if legal { 6 } else { 4 }
        );
    }
    let red = number(UnoColor::Red, 1);
    let (mut game, mut stats) = table(
        UnoRuleSet::default(),
        vec![
            vec![red, number(UnoColor::Blue, 2)],
            vec![number(UnoColor::Green, 2)],
        ],
        number(UnoColor::Red, 9),
    );
    record(
        &mut game,
        &mut stats,
        0,
        &[(red, None)],
        false,
        |game, player| game.play_card(player, red, None),
    );
    record(&mut game, &mut stats, 1, &[], false, |game, player| {
        game.report_uno(player, UnoPlayerId(0))
    });
    assert_eq!(progress(&stats, 1).uno_reports, 1);
    assert_eq!(progress(&stats, 0).uno_penalties, 1);
    assert!(progress(&stats, 0).has_penalty);
}
