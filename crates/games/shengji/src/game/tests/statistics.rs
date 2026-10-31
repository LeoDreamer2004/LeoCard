use super::dealt_game;
use crate::{
    ShengjiActionOutcome, ShengjiCard, ShengjiGameState, ShengjiGreedyBot, ShengjiGreedyBotRequest,
    ShengjiPhase, ShengjiRank, ShengjiRedealReason, ShengjiRuleSet, ShengjiSuit,
    ShengjiTeamProgress, ShengjiTrump, build_deck,
};
use leocard_game_common::PlayerId;

fn finish(game: &mut ShengjiGameState) {
    while game.phase() == &ShengjiPhase::Playing {
        let player = game.current_player().unwrap();
        let trick = game.current_trick();
        let play = ShengjiGreedyBot::choose(ShengjiGreedyBotRequest {
            hand: &game.players()[usize::from(player.0)].hand,
            lead: trick.as_ref().map(|trick| &trick.plays[0].1),
            trump: game.trump().unwrap(),
        })
        .unwrap();
        assert!(game.hand_statistics(player).is_none());
        game.play_cards(player, &play.cards).unwrap();
    }
}

#[test]
fn complete_opening_and_burial_facts_are_released_only_after_settlement() {
    let mut game = dealt_game(ShengjiRuleSet::default());
    let dealer = game.dealer().unwrap();
    let burial = game.players()[usize::from(dealer.0)].hand[..8].to_vec();
    assert!(game.hand_statistics(dealer).is_none());
    assert!(game.bury(dealer, &burial[..7]).is_err());
    assert!(game.statistics.burial[usize::from(dealer.0)].is_none());
    game.bury(dealer, &burial).unwrap();
    let trump = game.trump().unwrap();
    let expected = game
        .players()
        .iter()
        .map(|player| {
            (
                player.id,
                player
                    .hand
                    .iter()
                    .filter(|card| trump.is_trump(**card))
                    .count() as u8,
                player
                    .hand
                    .iter()
                    .filter(|card| {
                        matches!(card.rank(), ShengjiRank::SmallJoker | ShengjiRank::BigJoker)
                    })
                    .count() as u8,
            )
        })
        .collect::<Vec<_>>();
    finish(&mut game);
    for (player, trumps, jokers) in expected {
        assert!(game.players()[usize::from(player.0)].hand.is_empty());
        let facts = game.hand_statistics(player).unwrap();
        assert_eq!(facts.opening_hand.card_count, 25);
        assert_eq!(facts.opening_hand.trump_count, trumps);
        assert_eq!(facts.opening_hand.joker_count, jokers);
        if player == dealer {
            let buried = facts.burial.unwrap();
            assert_eq!(buried.card_count, 8);
            assert_eq!(buried.points, burial.iter().map(|card| card.points()).sum());
            assert_eq!(
                buried.scoring_card_count,
                burial.iter().filter(|card| card.points() > 0).count() as u8
            );
        } else {
            assert!(facts.burial.is_none());
        }
    }
    assert!(game.hand_statistics(PlayerId(4)).is_none());
}

#[test]
fn first_trick_cut_records_the_actual_cutter_even_if_overtrumped_later() {
    let mut game = ShengjiGameState::standard(build_deck()).unwrap();
    game.dealer = Some(PlayerId(0));
    game.trump = Some(ShengjiTrump::new(ShengjiRank::Two, Some(ShengjiSuit::Heart)).unwrap());
    let first = [
        ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Ten),
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Three),
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Four),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Ten),
    ];
    for (index, player) in game.players.iter_mut().enumerate() {
        player.hand = vec![
            first[index],
            ShengjiCard::suited(
                1,
                ShengjiSuit::Spade,
                [
                    ShengjiRank::Three,
                    ShengjiRank::Four,
                    ShengjiRank::Five,
                    ShengjiRank::Six,
                ][index],
            ),
        ];
    }
    game.start_playing();
    for (index, card) in first.into_iter().enumerate() {
        game.play_cards(PlayerId(index as u8), &[card]).unwrap();
    }
    assert_eq!(game.history()[0].winner, PlayerId(2));
    assert!(game.hand_statistics(PlayerId(1)).is_none());
    finish(&mut game);
    for player in 0..4 {
        assert_eq!(
            game.hand_statistics(PlayerId(player))
                .unwrap()
                .first_trick_cut_dealer,
            player == 1
        );
    }
}

#[test]
fn exhausted_bottom_redeal_retains_the_actual_power_outage_reason() {
    for power_outage_dealer in [false, true] {
        let rules = ShengjiRuleSet {
            power_outage_dealer,
            bottom_flip: true,
            ..ShengjiRuleSet::default()
        };
        // All copies of these four faces stay in the kitty, so nobody can match a reveal.
        let kitty = [
            ShengjiRank::Three,
            ShengjiRank::Four,
            ShengjiRank::Six,
            ShengjiRank::Seven,
        ]
        .into_iter()
        .flat_map(|rank| {
            (0..2).map(move |deck| ShengjiCard::suited(deck, ShengjiSuit::Diamond, rank))
        })
        .collect::<Vec<_>>();
        let mut deck = build_deck();
        deck.retain(|card| !kitty.contains(card));
        deck.extend(kitty);
        let mut game = ShengjiGameState::new(
            rules,
            ShengjiTeamProgress::default(),
            None,
            PlayerId(0),
            deck,
        )
        .unwrap();
        game.deal_all().unwrap();
        if power_outage_dealer {
            assert!(matches!(
                game.close_bidding_and_take_kitty().unwrap(),
                ShengjiActionOutcome::PowerOutageDealerChanged { .. }
            ));
        }
        assert_eq!(
            game.close_bidding_and_take_kitty().unwrap(),
            ShengjiActionOutcome::BottomFlipStarted
        );
        for _ in 0..8 {
            let ShengjiActionOutcome::BottomCardRevealed(reveal) =
                game.flip_next_bottom_card().unwrap()
            else {
                panic!("bottom reveal");
            };
            assert!(reveal.matches.is_empty());
        }
        assert_eq!(
            game.phase(),
            &ShengjiPhase::RedealRequired(ShengjiRedealReason::BottomFlipExhausted {
                power_outage_used: power_outage_dealer
            })
        );
        assert!(game.hand_statistics(PlayerId(0)).is_none());
    }
}
