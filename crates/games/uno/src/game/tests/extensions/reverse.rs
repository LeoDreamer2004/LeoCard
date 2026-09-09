use super::super::prelude::*;

fn reverse_pack_game() -> GameState {
    let rules = UnoRuleSet {
        reverse_pack: true,
        ..UnoRuleSet::default()
    };
    GameState::new_with_deck(rules, 6, build_deck_for_rules(rules)).unwrap()
}

#[test]
fn reverse_draw_two_matches_reverse_and_redirects_the_penalty() {
    let reverse_draw = card(UnoColor::Red, UnoFace::ReverseDrawTwo, 0);
    let filler = card(UnoColor::Blue, UnoFace::Number(3), 0);
    let mut game = reverse_pack_game();
    game.players[0].hand = vec![reverse_draw, filler];
    game.discard_pile = vec![card(UnoColor::Blue, UnoFace::Reverse, 0)];
    game.current_color = Some(UnoColor::Blue);
    game.current_player = UnoPlayerId(0);

    assert!(game.can_play(UnoPlayerId(0), reverse_draw));
    game.play_card(UnoPlayerId(0), reverse_draw, None).unwrap();

    let turn = game.turn().unwrap();
    assert_eq!(turn.direction, UnoDirection::CounterClockwise);
    assert_eq!(turn.pending_draw, 2);
    assert_eq!(turn.pending_kind, Some(UnoPendingDrawKind::DrawTwo));
    assert_eq!(turn.pending_draw_source, Some(UnoPlayerId(0)));
    assert_eq!(turn.current_player, UnoPlayerId(5));
}

#[test]
fn reverse_skip_changes_direction_before_choosing_the_skipped_player() {
    let reverse_skip = card(UnoColor::Red, UnoFace::ReverseSkip, 0);
    let filler = card(UnoColor::Blue, UnoFace::Number(3), 0);
    let mut game = reverse_pack_game();
    game.players[0].hand = vec![reverse_skip, filler];
    game.discard_pile = vec![card(UnoColor::Blue, UnoFace::Skip, 0)];
    game.current_color = Some(UnoColor::Blue);
    game.current_player = UnoPlayerId(0);

    assert!(game.can_play(UnoPlayerId(0), reverse_skip));
    game.play_card(UnoPlayerId(0), reverse_skip, None).unwrap();

    let turn = game.turn().unwrap();
    assert_eq!(turn.direction, UnoDirection::CounterClockwise);
    assert_eq!(turn.current_player, UnoPlayerId(5));
    assert_eq!(game.skipped_turns(UnoPlayerId(5)), Some(1));
}

#[test]
fn power_reverse_changes_color_and_gives_the_actor_another_turn() {
    let power = UnoCard::wild(UnoFace::WildPowerReverse, 0);
    let filler = card(UnoColor::Blue, UnoFace::Number(3), 0);
    let mut game = reverse_pack_game();
    game.players[0].hand = vec![power, filler];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(7), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    game.play_card(UnoPlayerId(0), power, Some(UnoColor::Green))
        .unwrap();

    let turn = game.turn().unwrap();
    assert_eq!(turn.direction, UnoDirection::CounterClockwise);
    assert_eq!(turn.current_color, Some(UnoColor::Green));
    assert_eq!(turn.current_player, UnoPlayerId(0));
}

#[test]
fn no_u_reflects_the_whole_penalty_to_the_latest_stacker() {
    let draw_a = card(UnoColor::Red, UnoFace::DrawTwo, 0);
    let draw_b = card(UnoColor::Blue, UnoFace::DrawTwo, 0);
    let no_u = UnoCard::wild(UnoFace::WildNoU, 0);
    let filler = card(UnoColor::Green, UnoFace::Number(3), 0);
    let mut game = reverse_pack_game();
    game.rules.action_stacking = true;
    game.players[0].hand = vec![draw_a, filler];
    game.players[1].hand = vec![draw_b, filler];
    game.players[2].hand = vec![no_u, filler];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(7), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    game.play_card(UnoPlayerId(0), draw_a, None).unwrap();
    game.play_card(UnoPlayerId(1), draw_b, None).unwrap();
    let before = game.player(UnoPlayerId(1)).unwrap().hand().len();
    let outcome = game
        .play_card(UnoPlayerId(2), no_u, Some(UnoColor::Yellow))
        .unwrap();

    assert!(matches!(
        outcome,
        ActionOutcome::Played {
            effect: Some(PlayedEffect::DrawReflected {
                player: UnoPlayerId(1),
                ref cards,
            }),
            ..
        } if cards.len() == 4
    ));
    assert_eq!(
        game.player(UnoPlayerId(1)).unwrap().hand().len(),
        before + 4
    );
    let turn = game.turn().unwrap();
    assert_eq!(turn.direction, UnoDirection::CounterClockwise);
    assert_eq!(turn.current_color, Some(UnoColor::Yellow));
    assert_eq!(turn.pending_draw, 0);
    assert_eq!(turn.pending_draw_source, None);
    assert_eq!(turn.challenge_offender, None);
    assert_eq!(turn.current_player, UnoPlayerId(0));
}

#[test]
fn reverse_skip_can_redirect_an_accumulated_skip() {
    let skip = card(UnoColor::Red, UnoFace::Skip, 0);
    let reverse_skip = card(UnoColor::Blue, UnoFace::ReverseSkip, 0);
    let filler = card(UnoColor::Green, UnoFace::Number(3), 0);
    let mut game = reverse_pack_game();
    game.rules.action_stacking = true;
    game.players[0].hand = vec![skip, filler];
    game.players[1].hand = vec![reverse_skip, filler];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(7), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    game.play_card(UnoPlayerId(0), skip, None).unwrap();
    assert!(game.can_play(UnoPlayerId(1), reverse_skip));
    game.play_card(UnoPlayerId(1), reverse_skip, None).unwrap();

    let turn = game.turn().unwrap();
    assert_eq!(turn.direction, UnoDirection::CounterClockwise);
    assert_eq!(turn.pending_skip, 2);
    assert_eq!(turn.current_player, UnoPlayerId(0));
}

#[test]
fn ordinary_skip_can_stack_on_reverse_skip() {
    let reverse_skip = card(UnoColor::Red, UnoFace::ReverseSkip, 0);
    let skip = card(UnoColor::Blue, UnoFace::Skip, 0);
    let filler = card(UnoColor::Green, UnoFace::Number(3), 0);
    let mut game = reverse_pack_game();
    game.rules.action_stacking = true;
    game.players[0].hand = vec![reverse_skip, filler];
    game.players[5].hand = vec![skip, filler];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(7), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    game.play_card(UnoPlayerId(0), reverse_skip, None).unwrap();
    assert!(game.can_play(UnoPlayerId(5), skip));
    game.play_card(UnoPlayerId(5), skip, None).unwrap();

    let turn = game.turn().unwrap();
    assert_eq!(turn.direction, UnoDirection::CounterClockwise);
    assert_eq!(turn.pending_skip, 2);
    assert_eq!(turn.current_player, UnoPlayerId(4));
}

#[test]
fn ordinary_draw_two_can_stack_on_reverse_draw_two() {
    let reverse_draw = card(UnoColor::Red, UnoFace::ReverseDrawTwo, 0);
    let draw_two = card(UnoColor::Blue, UnoFace::DrawTwo, 0);
    let filler = card(UnoColor::Green, UnoFace::Number(3), 0);
    let mut game = reverse_pack_game();
    game.rules.action_stacking = true;
    game.players[0].hand = vec![reverse_draw, filler];
    game.players[5].hand = vec![draw_two, filler];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(7), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    game.play_card(UnoPlayerId(0), reverse_draw, None).unwrap();
    assert!(game.can_play(UnoPlayerId(5), draw_two));
    game.play_card(UnoPlayerId(5), draw_two, None).unwrap();

    let turn = game.turn().unwrap();
    assert_eq!(turn.direction, UnoDirection::CounterClockwise);
    assert_eq!(turn.pending_draw, 4);
    assert_eq!(turn.pending_draw_source, Some(UnoPlayerId(5)));
    assert_eq!(turn.current_player, UnoPlayerId(4));
}

#[test]
fn last_no_u_reflects_the_pending_penalty_before_winning() {
    let no_u = UnoCard::wild(UnoFace::WildNoU, 0);
    let mut game = reverse_pack_game();
    game.rules.action_stacking = true;
    game.players[0].hand = vec![no_u];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::DrawTwo, 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);
    game.pending_draw = 2;
    game.pending_kind = Some(UnoPendingDrawKind::DrawTwo);
    game.pending_draw_source = Some(UnoPlayerId(5));
    let source_hand_len = game.player(UnoPlayerId(5)).unwrap().hand().len();

    assert!(matches!(
        game.play_card(UnoPlayerId(0), no_u, Some(UnoColor::Blue)),
        Ok(ActionOutcome::Played {
            effect: Some(PlayedEffect::DrawReflected {
                player: UnoPlayerId(5),
                ref cards,
            }),
            ..
        }) if cards.len() == 2
    ));
    assert_eq!(
        game.player(UnoPlayerId(5)).unwrap().hand().len(),
        source_hand_len + 2
    );
    assert!(matches!(
        game.phase(),
        Phase::Finished(GameResult {
            winner: UnoPlayerId(0),
            ..
        })
    ));
}
