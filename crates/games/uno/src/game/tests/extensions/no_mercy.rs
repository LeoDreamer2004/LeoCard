use crate::{UnoCard, UnoColor, UnoFace};
use leocard_game_common::PlayerId;

use super::super::prelude::*;

#[test]
fn no_mercy_draw_stack_only_accepts_an_equal_or_higher_value() {
    let draw_four = card(UnoColor::Red, UnoFace::DrawFour, 0);
    let draw_ten = UnoCard::wild(UnoFace::WildDrawTen, 0);
    let mut game = no_mercy_game(3);
    game.players[0].hand = vec![draw_four, draw_ten];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::DrawTwo, 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = PlayerId(0);
    game.pending_draw = 6;
    game.pending_kind = Some(UnoPendingDrawKind::NoMercy(6));
    game.pending_draw_source = Some(PlayerId(2));

    assert!(!game.can_play(PlayerId(0), draw_four));
    assert!(game.can_play(PlayerId(0), draw_ten));
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
    game.current_player = PlayerId(0);

    game.play_card(PlayerId(0), seven, None).unwrap();
    assert_eq!(
        game.pending_swap(),
        Some(UnoPendingSwap::SevenSwap {
            player: PlayerId(0)
        })
    );
    game.choose_seven_swap_target(PlayerId(0), PlayerId(1))
        .unwrap();
    assert_eq!(game.player(PlayerId(0)).unwrap().hand(), &[target]);
    assert_eq!(game.player(PlayerId(1)).unwrap().hand(), &[own]);
    assert_eq!(game.turn().unwrap().current_player, PlayerId(1));
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
    game.current_player = PlayerId(0);

    assert!(matches!(
        game.draw_card(PlayerId(0)),
        Ok(UnoActionOutcome::DrewCards {
            cards,
            playable: Some(card),
            next_player: PlayerId(0),
            ..
        }) if cards == vec![miss, playable] && card == playable
    ));
    assert_eq!(
        game.pass_after_draw(PlayerId(0)),
        Err(UnoGameError::MustPlayDrawnCard(playable))
    );
}

#[test]
fn no_mercy_eliminates_a_player_at_twenty_five_cards() {
    let mut game = no_mercy_game(3);
    game.players[0].hand = build_no_mercy_deck().into_iter().take(24).collect();
    game.current_player = PlayerId(0);
    game.pending_draw = 1;
    game.pending_kind = Some(UnoPendingDrawKind::NoMercy(2));
    game.pending_draw_source = Some(PlayerId(2));

    game.accept_draw_penalty(PlayerId(0)).unwrap();
    assert!(game.player(PlayerId(0)).unwrap().eliminated());
    assert!(game.player(PlayerId(0)).unwrap().hand().is_empty());
    assert_eq!(game.elimination_order, vec![PlayerId(0)]);
    assert_eq!(game.turn().unwrap().current_player, PlayerId(1));
}

#[test]
fn no_mercy_eliminations_take_last_places_without_hand_scores() {
    let mut game = no_mercy_game(5);
    game.players[0].hand.clear();
    game.players[1].eliminated = true;
    game.players[1].hand.clear();
    game.elimination_order.push(PlayerId(1));
    game.players[3].eliminated = true;
    game.players[3].hand.clear();
    game.elimination_order.push(PlayerId(3));
    game.players[2].hand = vec![card(UnoColor::Red, UnoFace::Number(5), 0)];
    game.players[4].hand = vec![card(UnoColor::Red, UnoFace::DrawTwo, 0)];

    let result = game.finish(PlayerId(0));

    assert_eq!(result.hand_scores, vec![0, 0, 5, 0, 20]);
    assert_eq!(result.placements, vec![1, 5, 2, 4, 3]);
    assert_eq!(result.reference_deltas, vec![4, -3, 1, -2, 0]);
}
