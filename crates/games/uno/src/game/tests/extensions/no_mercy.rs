use super::super::prelude::*;

#[test]
fn no_mercy_draw_stack_only_accepts_an_equal_or_higher_value() {
    let draw_four = card(UnoColor::Red, UnoFace::DrawFour, 0);
    let draw_ten = UnoCard::wild(UnoFace::WildDrawTen, 0);
    let mut game = no_mercy_game(3);
    game.players[0].hand = vec![draw_four, draw_ten];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::DrawTwo, 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);
    game.pending_draw = 6;
    game.pending_kind = Some(UnoPendingDrawKind::NoMercy(6));
    game.pending_draw_source = Some(UnoPlayerId(2));

    assert!(!game.can_play(UnoPlayerId(0), draw_four));
    assert!(game.can_play(UnoPlayerId(0), draw_ten));
}

#[test]
fn no_mercy_seven_swaps_the_actors_hand_with_one_target() {
    let seven = card(UnoColor::Red, UnoFace::Number(7), 0);
    let own = card(UnoColor::Blue, UnoFace::Number(1), 0);
    let target = card(UnoColor::Green, UnoFace::Number(2), 0);
    let mut game = no_mercy_game(3);
    game.players[0].hand = vec![seven, own];
    game.players[1].hand = vec![target];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(5), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    game.play_card(UnoPlayerId(0), seven, None).unwrap();
    assert_eq!(
        game.pending_swap(),
        Some(PendingSwap::SevenSwap {
            player: UnoPlayerId(0)
        })
    );
    game.choose_seven_swap_target(UnoPlayerId(0), UnoPlayerId(1))
        .unwrap();
    assert_eq!(game.player(UnoPlayerId(0)).unwrap().hand(), &[target]);
    assert_eq!(game.player(UnoPlayerId(1)).unwrap().hand(), &[own]);
    assert_eq!(game.turn().unwrap().current_player, UnoPlayerId(1));
}

#[test]
fn no_mercy_draws_until_a_playable_card() {
    let miss = card(UnoColor::Blue, UnoFace::Number(2), 0);
    let playable = card(UnoColor::Red, UnoFace::Number(3), 0);
    let mut game = no_mercy_game(3);
    game.players[0].hand = vec![card(UnoColor::Green, UnoFace::Number(1), 0)];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(5), 0)];
    game.draw_pile = [miss, playable].into();
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    assert!(matches!(
        game.draw_card(UnoPlayerId(0)),
        Ok(ActionOutcome::DrewCards {
            cards,
            playable: Some(card),
            next_player: UnoPlayerId(0),
            ..
        }) if cards == vec![miss, playable] && card == playable
    ));
    assert_eq!(
        game.pass_after_draw(UnoPlayerId(0)),
        Err(GameError::MustPlayDrawnCard(playable))
    );
}

#[test]
fn no_mercy_eliminates_a_player_at_twenty_five_cards() {
    let mut game = no_mercy_game(3);
    game.players[0].hand = build_no_mercy_deck().into_iter().take(24).collect();
    game.current_player = UnoPlayerId(0);
    game.pending_draw = 1;
    game.pending_kind = Some(UnoPendingDrawKind::NoMercy(2));
    game.pending_draw_source = Some(UnoPlayerId(2));

    game.accept_draw_penalty(UnoPlayerId(0)).unwrap();
    assert!(game.player(UnoPlayerId(0)).unwrap().eliminated());
    assert!(game.player(UnoPlayerId(0)).unwrap().hand().is_empty());
    assert_eq!(game.elimination_order, vec![UnoPlayerId(0)]);
    assert_eq!(game.turn().unwrap().current_player, UnoPlayerId(1));
}

#[test]
fn no_mercy_eliminations_take_last_places_without_hand_scores() {
    let mut game = no_mercy_game(5);
    game.players[0].hand.clear();
    game.players[1].eliminated = true;
    game.players[1].hand.clear();
    game.elimination_order.push(UnoPlayerId(1));
    game.players[3].eliminated = true;
    game.players[3].hand.clear();
    game.elimination_order.push(UnoPlayerId(3));
    game.players[2].hand = vec![card(UnoColor::Red, UnoFace::Number(5), 0)];
    game.players[4].hand = vec![card(UnoColor::Red, UnoFace::DrawTwo, 0)];

    let result = game.finish(UnoPlayerId(0));

    assert_eq!(result.hand_scores, vec![0, 0, 5, 0, 20]);
    assert_eq!(result.placements, vec![1, 5, 2, 4, 3]);
    assert_eq!(result.reference_deltas, vec![4, -3, 1, -2, 0]);
}

#[test]
fn sudden_death_only_stops_recycling_with_mercy_and_two_survivors() {
    let mut game = no_mercy_game(3);
    game.rules.no_mercy.sudden_death = true;
    game.draw_pile.clear();
    game.discard_pile = vec![
        card(UnoColor::Blue, UnoFace::Number(1), 0),
        card(UnoColor::Red, UnoFace::Number(5), 0),
    ];
    assert_eq!(game.draw_cards_for(UnoPlayerId(0), 1).unwrap().len(), 1);

    game.draw_pile.clear();
    game.players[2].eliminated = true;
    let discard = game.discard_pile.clone();
    assert!(game.draw_cards_for(UnoPlayerId(0), 1).unwrap().is_empty());
    assert_eq!(game.discard_pile, discard);

    game.rules.no_mercy.mercy_elimination = false;
    game.discard_pile
        .insert(0, card(UnoColor::Green, UnoFace::Number(3), 0));
    assert_eq!(game.draw_cards_for(UnoPlayerId(0), 1).unwrap().len(), 1);
}

#[test]
fn sudden_death_returns_turn_to_the_playing_player_after_a_miss() {
    let mut game = no_mercy_game(3);
    game.rules.no_mercy.sudden_death = true;
    game.players[2].eliminated = true;
    game.players[0].hand = vec![
        card(UnoColor::Red, UnoFace::Number(5), 0),
        card(UnoColor::Red, UnoFace::Number(6), 0),
        card(UnoColor::Green, UnoFace::Number(2), 0),
    ];
    game.players[1].hand = vec![card(UnoColor::Blue, UnoFace::Number(1), 0)];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(4), 0)];
    game.draw_pile.clear();
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);
    let first = game.players[0].hand[0];
    game.play_card(UnoPlayerId(0), first, None).unwrap();

    assert!(matches!(
        game.draw_card(UnoPlayerId(1)),
        Ok(ActionOutcome::DrewCards {
            cards,
            next_player: UnoPlayerId(0),
            ..
        }) if cards.is_empty()
    ));
    assert!(!game.turn().unwrap().sudden_death_free_play);
    let response = game.players[0].hand[0];
    assert!(game.can_play(UnoPlayerId(0), response));
    game.play_card(UnoPlayerId(0), response, None).unwrap();
    assert_eq!(game.sudden_death_misses, 0);
    assert_eq!(game.turn().unwrap().current_player, UnoPlayerId(1));
}

#[test]
fn sudden_death_allows_any_card_after_both_players_cannot_respond() {
    let mut game = no_mercy_game(3);
    game.rules.no_mercy.sudden_death = true;
    game.players[2].eliminated = true;
    game.players[0].hand = vec![
        card(UnoColor::Red, UnoFace::Number(5), 0),
        card(UnoColor::Green, UnoFace::Number(2), 0),
        card(UnoColor::Blue, UnoFace::Number(9), 0),
    ];
    game.players[1].hand = vec![card(UnoColor::Blue, UnoFace::Number(1), 0)];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(4), 0)];
    game.draw_pile.clear();
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);
    let first = game.players[0].hand[0];
    game.play_card(UnoPlayerId(0), first, None).unwrap();
    game.draw_card(UnoPlayerId(1)).unwrap();
    let unmatched = game.players[0].hand[0];
    assert!(!game.can_play(UnoPlayerId(0), unmatched));
    assert!(matches!(
        game.draw_card(UnoPlayerId(0)),
        Ok(ActionOutcome::DrewCards {
            cards,
            next_player: UnoPlayerId(0),
            ..
        }) if cards.is_empty()
    ));
    assert!(game.turn().unwrap().sudden_death_free_play);
    assert!(game.can_play(UnoPlayerId(0), unmatched));
    assert_eq!(
        game.draw_card(UnoPlayerId(0)),
        Err(GameError::DrawPileExhausted)
    );
    game.play_card(UnoPlayerId(0), unmatched, None).unwrap();
    assert!(!game.turn().unwrap().sudden_death_free_play);
    assert_eq!(game.turn().unwrap().current_player, UnoPlayerId(1));
}

#[test]
fn sudden_death_keeps_a_partial_final_draw_without_recycling() {
    let mut game = no_mercy_game(3);
    game.rules.no_mercy.sudden_death = true;
    game.players[2].eliminated = true;
    game.players[0].hand = vec![card(UnoColor::Green, UnoFace::Number(1), 0)];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(5), 0)];
    let last = card(UnoColor::Blue, UnoFace::Number(2), 0);
    game.draw_pile = [last].into();
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    assert!(matches!(
        game.draw_card(UnoPlayerId(0)),
        Ok(ActionOutcome::DrewCards {
            cards,
            next_player: UnoPlayerId(1),
            ..
        }) if cards == vec![last]
    ));
    assert_eq!(game.draw_pile_len(), 0);
    assert_eq!(game.discard_pile.len(), 1);
}

#[test]
fn sudden_death_settles_a_penalty_with_the_cards_still_in_the_pile() {
    let mut game = no_mercy_game(3);
    game.rules.no_mercy.sudden_death = true;
    game.players[2].eliminated = true;
    game.players[1].hand = vec![card(UnoColor::Green, UnoFace::Number(1), 0)];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::DrawFour, 0)];
    let last = card(UnoColor::Blue, UnoFace::Number(2), 0);
    game.draw_pile = [last].into();
    game.current_player = UnoPlayerId(1);
    game.pending_draw = 4;
    game.pending_kind = Some(UnoPendingDrawKind::NoMercy(4));
    game.pending_draw_source = Some(UnoPlayerId(0));

    assert!(matches!(
        game.accept_draw_penalty(UnoPlayerId(1)),
        Ok(ActionOutcome::PenaltyDrawn {
            cards,
            next_player: UnoPlayerId(0),
            ..
        }) if cards == vec![last]
    ));
    assert_eq!(game.draw_pile_len(), 0);
    assert_eq!(game.sudden_death_misses, 1);
    assert_eq!(game.discard_pile.len(), 1);
}
