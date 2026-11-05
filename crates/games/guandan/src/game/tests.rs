use super::{
    GuandanAction, GuandanActionOutcome, GuandanGameError, GuandanGameResult, GuandanGameState,
    GuandanPhase, GuandanTributeAction,
};
use crate::{GuandanCard, GuandanRuleSet, GuandanSuit, auto_play_action, build_deck};
use leocard_game_common::PlayerId;

fn card(rank: u8) -> GuandanCard {
    GuandanCard {
        deck: 0,
        suit: GuandanSuit::Spades,
        rank,
    }
}

#[test]
fn partners_finishing_first_and_second_ends_early_after_contact() {
    let mut game =
        GuandanGameState::new(GuandanRuleSet::default(), build_deck(), PlayerId(0)).unwrap();
    for (player, rank) in [2, 3, 4, 5].into_iter().enumerate() {
        game.players[player].hand = vec![card(rank)];
    }
    game.act(PlayerId(0), GuandanAction::Play(vec![card(2)]))
        .unwrap();
    game.act(PlayerId(1), GuandanAction::Pass).unwrap();
    game.act(PlayerId(2), GuandanAction::Pass).unwrap();
    let outcome = game.act(PlayerId(3), GuandanAction::Pass).unwrap();
    assert_eq!(
        outcome,
        GuandanActionOutcome::TrickCompleted {
            winner: PlayerId(0),
            next_player: PlayerId(2),
        }
    );
    assert!(game.last_actions().iter().all(Option::is_none));
    assert_eq!(
        game.previous_trick(),
        Some(&[
            Some(GuandanAction::Play(vec![card(2)])),
            Some(GuandanAction::Pass),
            Some(GuandanAction::Pass),
            Some(GuandanAction::Pass),
        ])
    );
    assert_eq!(game.finishing_order(), &[PlayerId(0)]);
    let outcome = game
        .act(PlayerId(2), GuandanAction::Play(vec![card(4)]))
        .unwrap();
    let GuandanActionOutcome::GameFinished(result) = outcome else {
        panic!("partners finishing first and second must end the hand")
    };
    assert_eq!(result.winning_team, 0);
    assert_eq!(result.level_gain, 3);
    assert_eq!(result.finishing_order[..2], [PlayerId(0), PlayerId(2)]);
}

#[test]
fn rejected_actions_leave_state_unchanged() {
    let mut game =
        GuandanGameState::new(GuandanRuleSet::default(), build_deck(), PlayerId(0)).unwrap();
    let initial = game.clone();
    assert_eq!(
        game.act(PlayerId(0), GuandanAction::Pass),
        Err(GuandanGameError::MustLeadWithCards)
    );
    assert_eq!(game, initial);
    let card = game.players[0].hand[0];
    assert_eq!(
        game.act(PlayerId(1), GuandanAction::Play(vec![card])),
        Err(GuandanGameError::NotPlayersTurn {
            expected: PlayerId(0),
            actual: PlayerId(1),
        })
    );
    assert_eq!(game, initial);
}

#[test]
fn two_tributes_return_to_play_with_27_cards_each() {
    let previous = GuandanGameResult {
        known_positions: 2,
        finishing_order: [PlayerId(0), PlayerId(2), PlayerId(1), PlayerId(3)],
        winning_team: 0,
        level_gain: 3,
    };
    let mut game = GuandanGameState::new_with_tribute(
        GuandanRuleSet::default(),
        build_deck(),
        PlayerId(0),
        Some(previous),
    )
    .unwrap();
    assert_eq!(game.tribute_pairs().len(), 2);
    assert!(!game.tribute_resisted());
    let giver = game.current_player().unwrap();
    let not_highest = game
        .player(giver)
        .unwrap()
        .hand()
        .iter()
        .copied()
        .find(|card| card.rank == 3)
        .unwrap();
    let initial = game.clone();
    assert_eq!(
        game.act(giver, GuandanAction::Tribute(not_highest)),
        Err(GuandanGameError::NotHighestTribute)
    );
    assert_eq!(game, initial);
    while let Some(action) = game.pending_tribute_action() {
        let player = game.current_player().unwrap();
        let hand = game.player(player).unwrap().hand();
        let card = match action {
            GuandanTributeAction::Tribute => {
                hand.iter().copied().max_by_key(|card| card.rank).unwrap()
            }
            GuandanTributeAction::ReturnTribute => hand
                .iter()
                .copied()
                .find(|card| card.rank <= 10 && Some(*card) != game.pending_offered_card())
                .unwrap(),
        };
        let command = match action {
            GuandanTributeAction::Tribute => GuandanAction::Tribute(card),
            GuandanTributeAction::ReturnTribute => GuandanAction::ReturnTribute(card),
        };
        if action == GuandanTributeAction::ReturnTribute {
            let offered = game.pending_offered_card().unwrap();
            assert_eq!(
                game.act(player, GuandanAction::ReturnTribute(offered)),
                Err(GuandanGameError::InvalidReturnTribute)
            );
        }
        game.act(player, command).unwrap();
    }
    assert_eq!(game.phase(), &GuandanPhase::Playing);
    assert_eq!(game.current_player(), Some(PlayerId(0)));
    assert!(
        game.players()
            .iter()
            .all(|player| player.hand().len() == 27)
    );
    assert!(game.last_actions().iter().all(Option::is_none));
}

#[test]
fn double_big_joker_in_a_giver_resists_tribute() {
    let mut deck = build_deck();
    deck.swap(105, 107);
    let previous = GuandanGameResult {
        known_positions: 2,
        finishing_order: [PlayerId(0), PlayerId(2), PlayerId(3), PlayerId(1)],
        winning_team: 0,
        level_gain: 3,
    };
    let game = GuandanGameState::new_with_tribute(
        GuandanRuleSet::default(),
        deck,
        PlayerId(0),
        Some(previous),
    )
    .unwrap();
    assert!(game.tribute_resisted());
    assert_eq!(game.phase(), &GuandanPhase::Playing);
    assert_eq!(game.pending_tribute_action(), None);
}

#[test]
fn four_automatic_players_can_complete_a_full_hand() {
    let mut game =
        GuandanGameState::new(GuandanRuleSet::default(), build_deck(), PlayerId(0)).unwrap();
    for _ in 0..500 {
        if matches!(game.phase(), GuandanPhase::Finished(_)) {
            return;
        }
        let player = game.current_player().unwrap();
        let action = auto_play_action(&game, player).expect("active bot has an action");
        game.act(player, action).expect("bot action is legal");
    }
    panic!("automatic players did not finish within 500 actions");
}
