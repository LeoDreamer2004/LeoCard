use super::dealt_game;
use crate::ShengjiTeamId;
use crate::{
    ShengjiAutoPlay, ShengjiAutoPlayRequest, ShengjiBot, ShengjiBotAction, ShengjiBotObservation,
    ShengjiCard, ShengjiGameState, ShengjiMode, ShengjiPhase, ShengjiRank, ShengjiRuleSet,
    ShengjiSuit, ShengjiTeamProgress, build_deck_for,
};
use leocard_game_common::{PlayerId, SearchBudget};

use std::time::Duration;

#[test]
fn six_players_deal_bury_follow_and_settle_a_complete_hand() {
    let rules = ShengjiRuleSet::for_mode(ShengjiMode::SixPlayer);
    assert_eq!(
        (rules.player_count(), rules.hand_size(), rules.kitty_size()),
        (6, 26, 6)
    );
    assert_eq!((rules.takeover_score(), rules.score_step()), (120, 60));
    assert!(
        ShengjiRuleSet {
            deck_count: 2,
            ..rules
        }
        .validate()
        .is_err()
    );
    assert!(
        ShengjiRuleSet {
            five_trump_crossing: true,
            ..rules
        }
        .validate()
        .is_err()
    );
    let mut game = dealt_game(rules);
    let dealer = game.dealer().unwrap();
    let buried = game.players()[usize::from(dealer.0)].hand[..6].to_vec();
    game.bury(dealer, &buried).unwrap();
    assert!(game.players().iter().all(|player| player.hand.len() == 26));
    let observation = ShengjiBotObservation::from_game(&game, dealer).unwrap();
    let action = ShengjiBot::choose(&observation, &SearchBudget::new(Duration::from_millis(10)));
    let ShengjiBotAction::Play(cards) = action else {
        panic!("bot must play after burial");
    };
    game.play_cards(dealer, &cards).unwrap();
    for _ in 0..156 {
        if matches!(game.phase(), ShengjiPhase::Finished(_)) {
            break;
        }
        let player = game.current_player().unwrap();
        let trick = game.current_trick();
        let lead = trick
            .as_ref()
            .and_then(|trick| trick.plays.first())
            .map(|(_, play)| play);
        let play = ShengjiAutoPlay::choose(ShengjiAutoPlayRequest {
            hand: &game.players()[usize::from(player.0)].hand,
            lead,
            trump: game.trump().unwrap(),
        })
        .unwrap();
        game.play_cards(player, &play.cards).unwrap();
    }
    let ShengjiPhase::Finished(result) = game.phase() else {
        panic!("hand did not finish");
    };
    assert!(game.players().iter().all(|player| player.hand.is_empty()));
    assert!(game.history().iter().all(|trick| trick.plays.len() == 6));
    let captured: u16 = game.history().iter().map(|trick| trick.points).sum();
    assert_eq!(
        captured + buried.iter().map(|card| card.points()).sum::<u16>(),
        300
    );
    assert!(result.next_dealer.0 < 6);
    assert_eq!(ShengjiTeamId::of(result.next_dealer), result.promoted_team);
}

#[test]
fn the_sixth_player_can_declare_and_bottom_copy_visits_all_seats() {
    let rules = ShengjiRuleSet {
        bottom_copy: true,
        ..ShengjiRuleSet::for_mode(ShengjiMode::SixPlayer)
    };
    let target = ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Two);
    let mut deck = build_deck_for(3);
    let index = deck.iter().position(|card| *card == target).unwrap();
    deck.swap(0, index);
    let mut game = ShengjiGameState::new(
        rules,
        ShengjiTeamProgress::default(),
        None,
        PlayerId(5),
        deck,
    )
    .unwrap();
    game.deal_next().unwrap();
    game.declare(PlayerId(5), &[target]).unwrap();
    game.deal_all().unwrap();
    game.close_bidding_and_take_kitty().unwrap();
    let buried = game.players()[5].hand[..6].to_vec();
    game.bury(PlayerId(5), &buried).unwrap();
    for expected in 0..5 {
        assert_eq!(
            game.bottom_copy().unwrap().current(),
            Some(PlayerId(expected))
        );
        game.choose_bottom_copy(PlayerId(expected), None).unwrap();
    }
    assert_eq!(game.phase(), &ShengjiPhase::Playing);
}
