use super::support::*;
use crate::UnoFace;

#[test]
fn pair_finishes_as_one_action_and_does_not_count_as_a_jump_in() {
    let first = card(UnoColor::Red, UnoFace::Number(7), 0);
    let second = card(UnoColor::Red, UnoFace::Number(7), 1);
    let rules = UnoRuleSet {
        jump_in: true,
        ..UnoRuleSet::default()
    };
    let (mut game, mut stats) = table(
        rules,
        vec![vec![first, second], vec![number(UnoColor::Blue, 9)]],
        number(UnoColor::Red, 5),
    );
    let facts = record(
        &mut game,
        &mut stats,
        0,
        &[(first, None), (second, None)],
        false,
        |game, player| game.play_cards(player, &[first, second], None),
    );
    let won = facts.iter().find(|facts| facts.won).unwrap();
    assert!(won.progress.finished_with_pair);
    assert_eq!(won.progress.jump_ins, 0);
    assert!(won.completed_game);
    assert!(won.progress.uno_calls == 0);
}

#[test]
fn numbers_and_colors_belong_to_one_game_and_exclude_chosen_wild_colors() {
    let red = (0..=9)
        .map(|value| number(UnoColor::Red, value))
        .collect::<Vec<_>>();
    let (mut game, mut stats) = table(
        UnoRuleSet::default(),
        vec![red.clone(), vec![number(UnoColor::Red, 9); 11]],
        number(UnoColor::Red, 0),
    );
    for (index, card) in red.iter().copied().enumerate() {
        record(
            &mut game,
            &mut stats,
            0,
            &[(card, None)],
            false,
            |game, player| game.play_card(player, card, None),
        );
        assert_eq!(progress(&stats, 0).numbers, (1 << (index + 1)) - 1);
        if index < 9 {
            let other = number(UnoColor::Red, 9);
            record(
                &mut game,
                &mut stats,
                1,
                &[(other, None)],
                false,
                |game, player| game.play_card(player, other, None),
            );
        }
    }
    assert_eq!(progress(&stats, 0).colors.count_ones(), 1);
    let wild = UnoCard::wild(UnoFace::Wild, 0);
    let (mut game, mut stats) = table(
        UnoRuleSet::default(),
        vec![vec![wild], vec![number(UnoColor::Red, 9)]],
        number(UnoColor::Red, 0),
    );
    record(
        &mut game,
        &mut stats,
        0,
        &[(wild, Some(UnoColor::Blue))],
        false,
        |game, player| game.play_card(player, wild, Some(UnoColor::Blue)),
    );
    assert_eq!(progress(&stats, 0).colors, 0);
    assert_eq!(progress(&stats, 0).wild_cards, 1);
}

#[test]
fn forgotten_uno_and_color_gifts_use_the_actual_finishing_play() {
    let red = number(UnoColor::Red, 9);
    let wild = UnoCard::wild(UnoFace::Wild, 0);
    let (mut game, mut stats) = table(
        UnoRuleSet::default(),
        vec![vec![wild, red], vec![number(UnoColor::Blue, 9)]],
        number(UnoColor::Blue, 1),
    );
    record(
        &mut game,
        &mut stats,
        0,
        &[(wild, Some(UnoColor::Blue))],
        false,
        |game, player| game.play_card(player, wild, Some(UnoColor::Blue)),
    );
    game.players[1].hand.push(number(UnoColor::Green, 1));
    let other = number(UnoColor::Blue, 9);
    record(
        &mut game,
        &mut stats,
        1,
        &[(other, None)],
        false,
        |game, player| game.play_card(player, other, None),
    );
    record(
        &mut game,
        &mut stats,
        0,
        &[(red, None)],
        false,
        |game, player| game.play_card(player, red, None),
    );
    assert!(progress(&stats, 0).finished_without_uno);
    assert!(!progress(&stats, 0).finished_with_color_gift);

    let blue = number(UnoColor::Blue, 1);
    let (mut game, mut stats) = table(
        UnoRuleSet::default(),
        vec![vec![wild, red], vec![blue]],
        number(UnoColor::Red, 2),
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
        &[(blue, None)],
        false,
        |game, player| game.play_card(player, blue, None),
    );
    assert!(progress(&stats, 1).finished_with_color_gift);
}

#[test]
fn only_the_no_mercy_zero_or_seven_initiator_receives_swap_credit() {
    let rules = UnoRuleSet {
        mode: Mode::NoMercy,
        ..UnoRuleSet::default()
    };
    for value in [0, 7] {
        let selected = number(UnoColor::Red, value);
        let (mut game, mut stats) = table(
            rules,
            vec![
                vec![selected, number(UnoColor::Blue, 1)],
                vec![number(UnoColor::Yellow, 2), number(UnoColor::Green, 3)],
            ],
            number(UnoColor::Red, 1),
        );
        record(
            &mut game,
            &mut stats,
            0,
            &[(selected, None)],
            false,
            |game, player| game.play_card(player, selected, None),
        );
        if value == 7 {
            assert_eq!(progress(&stats, 0).hand_swaps, 0);
            record(&mut game, &mut stats, 0, &[], false, |game, player| {
                game.choose_seven_swap_target(player, UnoPlayerId(1))
            });
        }
        assert_eq!(progress(&stats, 0).hand_swaps, 1);
        assert_eq!(progress(&stats, 1).hand_swaps, 0);
        assert!(!progress(&stats, 0).has_drawn);
    }
}

#[test]
fn a_deferred_palette_choice_can_give_the_next_player_the_winning_color() {
    let rules = UnoRuleSet {
        swap_pack: true,
        ..UnoRuleSet::default()
    };
    let trade = UnoCard::wild(UnoFace::WildForceTrade, 0);
    let red = number(UnoColor::Red, 9);
    let (mut game, mut stats) = table(
        rules,
        vec![vec![trade, red], vec![number(UnoColor::Blue, 1)]],
        number(UnoColor::Blue, 2),
    );
    record(
        &mut game,
        &mut stats,
        0,
        &[(trade, None)],
        false,
        |game, player| game.play_card(player, trade, None),
    );
    record(&mut game, &mut stats, 0, &[], false, |game, player| {
        game.force_trade_hands(player, player, UnoPlayerId(1))
    });
    record(&mut game, &mut stats, 0, &[], false, |game, player| {
        game.choose_initial_color(player, UnoColor::Red)
    });
    record(
        &mut game,
        &mut stats,
        1,
        &[(red, None)],
        false,
        |game, player| game.play_card(player, red, None),
    );
    assert!(progress(&stats, 1).finished_with_color_gift);
}
