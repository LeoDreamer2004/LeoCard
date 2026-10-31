use crate::{UnoCard, UnoColor, UnoFace, UnoRuleSet, build_deck_for_rules};
use leocard_game_common::PlayerId;

use super::super::prelude::*;

fn swap_pack_game() -> UnoGameState {
    let rules = UnoRuleSet {
        swap_pack: true,
        ..UnoRuleSet::default()
    };
    UnoGameState::new_with_deck(rules, 6, build_deck_for_rules(rules)).unwrap()
}

#[test]
fn refresh_hand_places_old_cards_under_the_discard_and_draws_the_same_count() {
    let refresh = card(UnoColor::Red, UnoFace::RefreshHand, 0);
    let first = card(UnoColor::Blue, UnoFace::Number(1), 0);
    let second = card(UnoColor::Green, UnoFace::Number(2), 0);
    let top = card(UnoColor::Red, UnoFace::Number(5), 0);
    let mut game = swap_pack_game();
    game.players[0].hand = vec![refresh, first, second];
    game.discard_pile = vec![top];
    game.current_color = Some(UnoColor::Red);
    game.current_player = PlayerId(0);
    game.rules.action_stacking = true;
    game.rules.jump_in = true;

    let outcome = game.play_card(PlayerId(0), refresh, None).unwrap();

    assert!(matches!(
        outcome,
        UnoActionOutcome::Played {
            effect: Some(UnoPlayedEffect::HandRefreshed { count: 2 }),
            ..
        }
    ));
    assert_eq!(game.player(PlayerId(0)).unwrap().hand().len(), 2);
    assert_eq!(game.discard_pile(), &[first, second, top, refresh]);
    assert_eq!(game.top_card(), refresh);
    assert!(!game.jump_in_open);
}

#[test]
fn swap_one_allows_returning_the_taken_card_without_creating_uno_state() {
    let swap = card(UnoColor::Red, UnoFace::SwapOne, 0);
    let filler = card(UnoColor::Blue, UnoFace::Number(1), 0);
    let taken = card(UnoColor::Yellow, UnoFace::Number(4), 0);
    let target_filler = card(UnoColor::Green, UnoFace::Number(6), 0);
    let mut game = swap_pack_game();
    game.players[0].hand = vec![swap, filler];
    game.players[1].hand = vec![taken, target_filler];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(5), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = PlayerId(0);

    game.play_card(PlayerId(0), swap, None).unwrap();
    game.choose_swap_one_target(PlayerId(0), PlayerId(1), 0)
        .unwrap();
    assert_eq!(game.player(PlayerId(1)).unwrap().hand().len(), 1);
    assert!(game.uno_exposed_players().next().is_none());
    assert!(game.player(PlayerId(0)).unwrap().hand().contains(&taken));

    game.give_swap_one_card(PlayerId(0), taken).unwrap();
    assert_eq!(game.turn().unwrap().current_player, PlayerId(1));
    assert_eq!(game.player(PlayerId(1)).unwrap().hand().len(), 2);
    assert!(game.player(PlayerId(1)).unwrap().hand().contains(&taken));
    assert!(
        !game
            .uno_exposed_players()
            .any(|player| player == PlayerId(1))
    );
}

#[test]
fn force_trade_may_include_actor_and_defers_color_until_after_confirmation() {
    let trade = UnoCard::wild(UnoFace::WildForceTrade, 0);
    let actor_card = card(UnoColor::Blue, UnoFace::Number(1), 0);
    let target_card = card(UnoColor::Yellow, UnoFace::Number(2), 0);
    let mut game = swap_pack_game();
    game.players[0].hand = vec![trade, actor_card];
    game.players[1].hand = vec![target_card];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(5), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = PlayerId(0);
    game.uno_exposed[1] = true;

    game.play_card(PlayerId(0), trade, None).unwrap();
    assert_eq!(game.current_color(), None);
    assert_eq!(
        game.choose_initial_color(PlayerId(0), UnoColor::Green),
        Err(UnoGameError::MustResolveSwapEffect)
    );
    game.force_trade_hands(PlayerId(0), PlayerId(0), PlayerId(1))
        .unwrap();
    assert_eq!(game.player(PlayerId(0)).unwrap().hand(), &[target_card]);
    assert_eq!(game.player(PlayerId(1)).unwrap().hand(), &[actor_card]);
    assert!(game.uno_exposed_players().next().is_none());

    game.choose_initial_color(PlayerId(0), UnoColor::Green)
        .unwrap();
    assert_eq!(game.current_color(), Some(UnoColor::Green));
    assert_eq!(game.turn().unwrap().current_player, PlayerId(1));
}

#[test]
fn pass_hands_follows_direction_and_clears_every_uno_state() {
    let pass = UnoCard::wild(UnoFace::WildPassHands, 0);
    let mut game = swap_pack_game();
    let markers = [
        card(UnoColor::Red, UnoFace::Number(1), 0),
        card(UnoColor::Yellow, UnoFace::Number(2), 0),
        card(UnoColor::Green, UnoFace::Number(3), 0),
        card(UnoColor::Blue, UnoFace::Number(4), 0),
        card(UnoColor::Red, UnoFace::Number(5), 0),
        card(UnoColor::Yellow, UnoFace::Number(6), 0),
    ];
    game.players[0].hand = vec![pass, markers[0]];
    for (index, marker) in markers.iter().copied().enumerate().skip(1) {
        game.players[index].hand = vec![marker];
        game.uno_declared[index] = true;
        game.uno_exposed[index] = true;
    }
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(9), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = PlayerId(0);

    game.play_card(PlayerId(0), pass, None).unwrap();

    for (source, marker) in markers.iter().copied().enumerate() {
        let target = (source + 1) % markers.len();
        assert_eq!(
            game.player(PlayerId::from_index(target)).unwrap().hand(),
            &[marker]
        );
    }
    assert!(game.uno_declared_players().next().is_none());
    assert!(game.uno_exposed_players().next().is_none());
    assert_eq!(
        game.pending_swap(),
        Some(UnoPendingSwap::ChooseColor {
            player: PlayerId(0)
        })
    );
}

#[test]
fn no_mercy_zero_pass_skips_eliminated_players() {
    let zero = card(UnoColor::Red, UnoFace::Number(0), 0);
    let markers = [
        card(UnoColor::Blue, UnoFace::Number(1), 0),
        card(UnoColor::Green, UnoFace::Number(2), 0),
        card(UnoColor::Yellow, UnoFace::Number(3), 0),
    ];
    let mut game = no_mercy_game(4);
    game.players[0].hand = vec![zero, markers[0]];
    game.players[1].hand = vec![markers[1]];
    game.players[2].hand.clear();
    game.players[2].eliminated = true;
    game.players[3].hand = vec![markers[2]];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(5), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = PlayerId(0);

    game.play_card(PlayerId(0), zero, None).unwrap();

    assert_eq!(game.player(PlayerId(0)).unwrap().hand(), &[markers[2]]);
    assert_eq!(game.player(PlayerId(1)).unwrap().hand(), &[markers[0]]);
    assert!(game.player(PlayerId(2)).unwrap().hand().is_empty());
    assert_eq!(game.player(PlayerId(3)).unwrap().hand(), &[markers[1]]);
}

#[test]
fn hand_trade_targets_cannot_be_eliminated() {
    let mut game = swap_pack_game();
    game.pending_swap = Some(PendingSwapState::ForceTrade {
        player: PlayerId(0),
    });
    game.current_player = PlayerId(0);
    game.players[2].hand.clear();
    game.players[2].eliminated = true;

    assert_eq!(
        game.force_trade_hands(PlayerId(0), PlayerId(1), PlayerId(2)),
        Err(UnoGameError::InvalidSwapTargets)
    );
}

#[test]
fn last_swap_pack_card_runs_its_effect_before_winning() {
    let refresh = card(UnoColor::Red, UnoFace::RefreshHand, 0);
    let top = card(UnoColor::Red, UnoFace::Number(5), 0);
    let mut game = swap_pack_game();
    game.players[0].hand = vec![refresh];
    game.discard_pile = vec![top];
    game.current_color = Some(UnoColor::Red);
    game.current_player = PlayerId(0);

    assert!(matches!(
        game.play_card(PlayerId(0), refresh, None),
        Ok(UnoActionOutcome::Played {
            effect: Some(UnoPlayedEffect::HandRefreshed { count: 0 }),
            ..
        })
    ));
    assert_eq!(game.discard_pile(), &[top, refresh]);
    assert_eq!(game.pending_swap(), None);
    assert!(matches!(
        game.phase(),
        UnoPhase::Finished(UnoGameResult {
            winner: PlayerId(0),
            ..
        })
    ));
}

#[test]
fn swap_pack_starting_cards_do_not_run_effects_and_wild_still_chooses_color() {
    let rules = UnoRuleSet {
        swap_pack: true,
        ..UnoRuleSet::default()
    };
    let colored = card(UnoColor::Red, UnoFace::SwapOne, 0);
    let mut deck = build_deck_for_rules(rules);
    let index = deck.iter().position(|card| *card == colored).unwrap();
    deck.swap(42, index);
    let game = UnoGameState::new_with_deck(rules, 6, deck).unwrap();
    assert_eq!(game.top_card(), colored);
    assert_eq!(game.current_color(), Some(UnoColor::Red));
    assert_eq!(game.pending_swap(), None);
    assert_eq!(game.turn().unwrap().current_player, PlayerId(0));

    let wild = UnoCard::wild(UnoFace::WildForceTrade, 0);
    let mut deck = build_deck_for_rules(rules);
    let index = deck.iter().position(|card| *card == wild).unwrap();
    deck.swap(42, index);
    let mut game = UnoGameState::new_with_deck(rules, 6, deck).unwrap();
    assert_eq!(game.top_card(), wild);
    assert_eq!(game.current_color(), None);
    assert_eq!(game.pending_swap(), None);
    game.choose_initial_color(PlayerId(0), UnoColor::Blue)
        .unwrap();
    assert_eq!(game.current_color(), Some(UnoColor::Blue));
    assert_eq!(game.turn().unwrap().current_player, PlayerId(0));
}
