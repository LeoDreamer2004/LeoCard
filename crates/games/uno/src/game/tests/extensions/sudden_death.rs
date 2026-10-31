use crate::{UnoCard, UnoColor, UnoFace};
use leocard_game_common::PlayerId;

use super::super::prelude::*;

#[test]
fn players_can_decline_playable_responses_without_drawing_or_recycling() {
    let mut game = no_mercy_game(3);
    game.rules.no_mercy.sudden_death = true;
    game.players[2].eliminated = true;
    let first = card(UnoColor::Red, UnoFace::Number(5), 0);
    let second = UnoCard::wild(UnoFace::WildDrawSix, 0);
    game.players[0].hand = vec![first, card(UnoColor::Green, UnoFace::Number(2), 0)];
    game.players[1].hand = vec![second, card(UnoColor::Blue, UnoFace::Number(1), 0)];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(4), 0)];
    game.draw_pile.clear();
    game.current_color = Some(UnoColor::Red);
    game.current_player = PlayerId(0);
    let hands: Vec<_> = game
        .players
        .iter()
        .map(|player| player.hand.clone())
        .collect();
    let discard = game.discard_pile.clone();

    for (player, response) in [(PlayerId(0), first), (PlayerId(1), second)] {
        assert!(game.can_play(player, response));
        assert!(matches!(
            game.draw_card(player),
            Ok(UnoActionOutcome::DrewCards { cards, playable: None, .. }) if cards.is_empty()
        ));
    }

    assert_eq!(game.turn().unwrap().current_player, PlayerId(1));
    assert!(game.turn().unwrap().sudden_death_free_play);
    assert_eq!(game.discard_pile, discard);
    assert_eq!(game.draw_pile_len(), 0);
    for (player, hand) in game.players.iter().zip(hands) {
        assert_eq!(player.hand, hand);
    }
    assert_eq!(
        game.draw_card(PlayerId(1)),
        Err(UnoGameError::DrawPileExhausted)
    );
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
    assert_eq!(game.draw_cards_for(PlayerId(0), 1).unwrap().len(), 1);

    game.draw_pile.clear();
    game.players[2].eliminated = true;
    let discard = game.discard_pile.clone();
    assert!(game.draw_cards_for(PlayerId(0), 1).unwrap().is_empty());
    assert_eq!(game.discard_pile, discard);

    game.rules.no_mercy.mercy_elimination = false;
    game.discard_pile
        .insert(0, card(UnoColor::Green, UnoFace::Number(3), 0));
    assert_eq!(game.draw_cards_for(PlayerId(0), 1).unwrap().len(), 1);
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
    game.current_player = PlayerId(0);
    let first = game.players[0].hand[0];
    game.play_card(PlayerId(0), first, None).unwrap();

    assert!(matches!(
        game.draw_card(PlayerId(1)),
        Ok(UnoActionOutcome::DrewCards {
            cards,
            next_player: PlayerId(0),
            ..
        }) if cards.is_empty()
    ));
    assert!(!game.turn().unwrap().sudden_death_free_play);
    let response = game.players[0].hand[0];
    assert!(game.can_play(PlayerId(0), response));
    game.play_card(PlayerId(0), response, None).unwrap();
    assert_eq!(game.sudden_death_misses, 0);
    assert_eq!(game.turn().unwrap().current_player, PlayerId(1));
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
    game.current_player = PlayerId(0);
    let first = game.players[0].hand[0];
    game.play_card(PlayerId(0), first, None).unwrap();
    game.draw_card(PlayerId(1)).unwrap();
    let unmatched = game.players[0].hand[0];
    assert!(!game.can_play(PlayerId(0), unmatched));
    assert!(matches!(
        game.draw_card(PlayerId(0)),
        Ok(UnoActionOutcome::DrewCards {
            cards,
            next_player: PlayerId(0),
            ..
        }) if cards.is_empty()
    ));
    assert!(game.turn().unwrap().sudden_death_free_play);
    assert!(game.can_play(PlayerId(0), unmatched));
    assert_eq!(
        game.draw_card(PlayerId(0)),
        Err(UnoGameError::DrawPileExhausted)
    );
    game.play_card(PlayerId(0), unmatched, None).unwrap();
    assert!(!game.turn().unwrap().sudden_death_free_play);
    assert_eq!(game.turn().unwrap().current_player, PlayerId(1));
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
    game.current_player = PlayerId(0);

    assert!(matches!(
        game.draw_card(PlayerId(0)),
        Ok(UnoActionOutcome::DrewCards {
            cards,
            next_player: PlayerId(1),
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
    game.current_player = PlayerId(1);
    game.pending_draw = 4;
    game.pending_kind = Some(UnoPendingDrawKind::NoMercy(4));
    game.pending_draw_source = Some(PlayerId(0));

    assert!(matches!(
        game.accept_draw_penalty(PlayerId(1)),
        Ok(UnoActionOutcome::PenaltyDrawn {
            cards,
            next_player: PlayerId(0),
            ..
        }) if cards == vec![last]
    ));
    assert_eq!(game.draw_pile_len(), 0);
    assert_eq!(game.sudden_death_misses, 1);
    assert_eq!(game.discard_pile.len(), 1);
}
