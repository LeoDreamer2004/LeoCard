use super::dealt_game;
use crate::{
    ActionOutcome, GameState, Phase, ShengjiCard as Card, ShengjiGreedyBot,
    ShengjiGreedyBotRequest, ShengjiPlayerId as Player, ShengjiRank as Rank, ShengjiRedealReason,
    ShengjiRuleSet, ShengjiSuit as Suit, ShengjiTrump, TeamProgress, build_deck,
};

fn finish(game: &mut GameState) {
    while game.phase() == &Phase::Playing {
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
                    .filter(|card| matches!(card.rank(), Rank::SmallJoker | Rank::BigJoker))
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
    assert!(game.hand_statistics(Player(4)).is_none());
}

#[test]
fn first_trick_cut_records_the_actual_cutter_even_if_overtrumped_later() {
    let mut game = GameState::standard(build_deck()).unwrap();
    game.dealer = Some(Player(0));
    game.trump = Some(ShengjiTrump::new(Rank::Two, Some(Suit::Heart)).unwrap());
    let first = [
        Card::suited(0, Suit::Diamond, Rank::Ten),
        Card::suited(0, Suit::Heart, Rank::Three),
        Card::suited(0, Suit::Heart, Rank::Four),
        Card::suited(0, Suit::Club, Rank::Ten),
    ];
    for (index, player) in game.players.iter_mut().enumerate() {
        player.hand = vec![
            first[index],
            Card::suited(
                1,
                Suit::Spade,
                [Rank::Three, Rank::Four, Rank::Five, Rank::Six][index],
            ),
        ];
    }
    game.start_playing();
    for (index, card) in first.into_iter().enumerate() {
        game.play_cards(Player(index as u8), &[card]).unwrap();
    }
    assert_eq!(game.history()[0].winner, Player(2));
    assert!(game.hand_statistics(Player(1)).is_none());
    finish(&mut game);
    for player in 0..4 {
        assert_eq!(
            game.hand_statistics(Player(player))
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
        let kitty = [Rank::Three, Rank::Four, Rank::Six, Rank::Seven]
            .into_iter()
            .flat_map(|rank| (0..2).map(move |deck| Card::suited(deck, Suit::Diamond, rank)))
            .collect::<Vec<_>>();
        let mut deck = build_deck();
        deck.retain(|card| !kitty.contains(card));
        deck.extend(kitty);
        let mut game =
            GameState::new(rules, TeamProgress::default(), None, Player(0), deck).unwrap();
        game.deal_all().unwrap();
        if power_outage_dealer {
            assert!(matches!(
                game.close_bidding_and_take_kitty().unwrap(),
                ActionOutcome::PowerOutageDealerChanged { .. }
            ));
        }
        assert_eq!(
            game.close_bidding_and_take_kitty().unwrap(),
            ActionOutcome::BottomFlipStarted
        );
        for _ in 0..8 {
            let ActionOutcome::BottomCardRevealed(reveal) = game.flip_next_bottom_card().unwrap()
            else {
                panic!("bottom reveal");
            };
            assert!(reveal.matches.is_empty());
        }
        assert_eq!(
            game.phase(),
            &Phase::RedealRequired(ShengjiRedealReason::BottomFlipExhausted {
                power_outage_used: power_outage_dealer
            })
        );
        assert!(game.hand_statistics(Player(0)).is_none());
    }
}
