use super::support::*;

fn own_turn(game: &mut GameState, stats: &mut UnoMatchStatistics, call: bool) {
    record(game, stats, 0, &[], false, GameState::draw_card);
    if call {
        record(game, stats, 0, &[], false, GameState::call_uno);
    }
    let drawn = card(UnoColor::Red, UnoFace::Number(9), 1);
    record(game, stats, 0, &[(drawn, None)], false, |game, player| {
        game.play_card(player, drawn, None)
    });
    let other = number(UnoColor::Red, 9);
    record(game, stats, 1, &[(other, None)], false, |game, player| {
        game.play_card(player, other, None)
    });
}

#[test]
fn consecutive_calls_reset_on_a_normal_turn_without_a_call() {
    let (mut game, mut stats) = table(
        UnoRuleSet::default(),
        vec![
            vec![number(UnoColor::Red, 9)],
            vec![number(UnoColor::Red, 9); 30],
        ],
        number(UnoColor::Red, 1),
    );
    game.draw_pile = vec![card(UnoColor::Red, UnoFace::Number(9), 1); 30].into();
    for _ in 0..9 {
        own_turn(&mut game, &mut stats, true);
    }
    assert_eq!(progress(&stats, 0).max_uno_call_run, 9);
    own_turn(&mut game, &mut stats, false);
    for _ in 0..9 {
        own_turn(&mut game, &mut stats, true);
    }
    assert_eq!(progress(&stats, 0).max_uno_call_run, 9);
    own_turn(&mut game, &mut stats, true);
    assert_eq!(progress(&stats, 0).max_uno_call_run, 10);
    assert_eq!(progress(&stats, 0).uno_calls, 19);
}

#[test]
fn recovering_uno_during_someone_elses_turn_does_not_extend_the_run() {
    let red = number(UnoColor::Red, 9);
    let (mut game, mut stats) = table(
        UnoRuleSet::default(),
        vec![
            vec![red, number(UnoColor::Blue, 2)],
            vec![number(UnoColor::Green, 2)],
        ],
        number(UnoColor::Red, 1),
    );
    record(
        &mut game,
        &mut stats,
        0,
        &[(red, None)],
        false,
        |game, player| game.play_card(player, red, None),
    );
    record(&mut game, &mut stats, 0, &[], false, GameState::call_uno);
    assert_eq!(progress(&stats, 0).uno_calls, 1);
    assert_eq!(progress(&stats, 0).max_uno_call_run, 0);
}
