mod match_state;

use crate::*;

fn cards(ranks: &[u8]) -> Vec<Card> {
    let mut used = [0; 15];
    ranks
        .iter()
        .map(|&r| {
            let n = &mut used[usize::from(r)];
            let card = if r < 13 {
                Card((*n / 4) * 54 + r * 4 + *n % 4)
            } else {
                Card(*n * 54 + 52 + r - 13)
            };
            *n += 1;
            card
        })
        .collect()
}

#[test]
fn standard_patterns_and_bomb_order() {
    for (ranks, kind) in [
        (vec![0, 1, 2, 3, 4], PlayKind::Straight),
        (vec![0, 0, 1, 1, 2, 2], PlayKind::Pairs),
        (vec![1, 1, 1, 2, 2, 2, 5, 6], PlayKind::AirplaneSingles),
        (vec![1, 1, 1, 2, 2, 2, 5, 5, 6, 6], PlayKind::AirplanePairs),
        (vec![3, 3, 3, 3, 5, 6], PlayKind::FourSingles),
    ] {
        assert_eq!(classify(&cards(&ranks), Mode::Three).unwrap().kind, kind);
    }
    assert!(classify(&cards(&[8, 9, 10, 11, 12]), Mode::Three).is_none());
    assert!(classify(&[Card(0), Card(0)], Mode::Three).is_none());
    let bomb = classify(&cards(&[0; 5]), Mode::Four).unwrap();
    assert!(bomb.beats(&classify(&cards(&[12; 4]), Mode::Four).unwrap()));
    let rocket = classify(&cards(&[13, 13, 14, 14]), Mode::Four).unwrap();
    assert_eq!(rocket.kind, PlayKind::Rocket);
    assert!(rocket.beats(&bomb));
    assert!(classify(&cards(&[13, 14]), Mode::Four).is_none());
}

#[test]
fn hints_offer_distinct_legal_responses_in_bot_preference_order() {
    let hand = cards(&[1, 1, 2, 3, 3, 3, 3, 5]);
    let top = classify(&cards(&[0]), Mode::Three).unwrap();
    let suggestions = hint_options(&hand, Some(&top), Mode::Three);
    assert!(suggestions.len() > 1);
    assert_eq!(
        hint(&hand, Some(&top), Mode::Three),
        suggestions.first().cloned()
    );
    for (index, play) in suggestions.iter().enumerate() {
        assert!(play.beats(&top));
        assert!(play.cards.iter().all(|card| hand.contains(card)));
        assert!(
            suggestions[..index]
                .iter()
                .all(|previous| previous.cards != play.cards)
        );
    }
}

#[test]
fn deals_bidding_privacy_and_redeal() {
    for mode in [Mode::Three, Mode::Four] {
        let mut game = GameState::new(
            RuleSet {
                mode,
                ..Default::default()
            },
            build_deck(mode),
            0,
        )
        .unwrap();
        let hand_len = if mode == Mode::Three { 17 } else { 25 };
        assert!(game.hand_lengths().iter().all(|&n| n == hand_len));
        assert!(game.public_kitty().is_empty());
        assert_eq!(game.act(1, Action::Bid(1)), Err(GameError::NotYourTurn));
        game.act(0, Action::Bid(1)).unwrap();
        assert!(game.public_kitty().is_empty());
        assert_eq!(game.act(1, Action::Bid(1)), Err(GameError::InvalidBid));
        game.act(1, Action::Bid(3)).unwrap();
        assert_eq!(game.phase, Phase::Playing);
        assert_eq!(game.current, 1);
        assert_eq!(
            game.hand(1).unwrap().len(),
            usize::from(hand_len) + mode.kitty_len()
        );
        assert_eq!(game.act(1, Action::Pass), Err(GameError::MustPlay));
        let card = game.hand(1).unwrap()[0];
        game.act(1, Action::Play(vec![card])).unwrap();
        for _ in 0..mode.players() - 1 {
            game.act(game.current, Action::Pass).unwrap();
        }
        assert!(game.table.is_none());
        assert_eq!(game.current, 1);
        assert!(game.last_actions.iter().all(Option::is_none));
        let mut game = GameState::new(
            RuleSet {
                mode,
                ..Default::default()
            },
            build_deck(mode),
            0,
        )
        .unwrap();
        for _ in 0..mode.players() {
            game.act(game.current, Action::Bid(0)).unwrap();
        }
        assert_eq!(game.phase, Phase::Redeal);
    }
}

#[test]
fn bots_complete_both_modes_and_settle_zero_sum() {
    for mode in [Mode::Three, Mode::Four] {
        for first in 0..mode.players() {
            let mut game = GameState::new(
                RuleSet {
                    mode,
                    ..Default::default()
                },
                build_deck(mode),
                first,
            )
            .unwrap();
            game.act(first, Action::Bid(3)).unwrap();
            for turn in 0..2000 {
                if game.phase == Phase::Finished {
                    break;
                }
                let action = if turn % 2 == 0 {
                    bot_action(&game)
                } else {
                    auto_play_action(&game)
                }
                .unwrap();
                game.act(game.current, action).unwrap();
            }
            assert_eq!(game.phase, Phase::Finished);
            assert_eq!(game.result.as_ref().unwrap().scores.iter().sum::<i32>(), 0);
            assert_eq!(
                game.played.len()
                    + game
                        .hand_lengths()
                        .iter()
                        .map(|&n| usize::from(n))
                        .sum::<usize>(),
                build_deck(mode).len()
            );
        }
    }
}

#[test]
fn spring_and_reverse_spring_double_once_and_settle_all_farmers() {
    for mode in [Mode::Three, Mode::Four] {
        for winner in [0, 1] {
            let mut game = GameState::new(
                RuleSet {
                    mode,
                    ..Default::default()
                },
                build_deck(mode),
                0,
            )
            .unwrap();
            game.act(0, Action::Bid(3)).unwrap();
            if winner == 1 {
                let card = game.hand(0).unwrap()[0];
                game.act(0, Action::Play(vec![card])).unwrap();
            }
            let mut before_finish = 1;
            for _ in 0..500 {
                if game.phase == Phase::Finished {
                    break;
                }
                let action = if game.current == winner {
                    Action::Play(
                        hint(
                            game.hand(winner).unwrap(),
                            game.table.as_ref().map(|(_, p)| p),
                            mode,
                        )
                        .unwrap()
                        .cards,
                    )
                } else {
                    Action::Pass
                };
                before_finish = game.multiplier;
                if let Action::Play(cards) = &action
                    && classify(cards, mode).unwrap().doubles(mode)
                {
                    before_finish *= 2;
                }
                game.act(game.current, action).unwrap();
            }
            let result = game
                .result
                .as_ref()
                .expect("the designated player finishes");
            assert!(result.spring);
            assert_eq!(result.landlord_won, winner == 0);
            assert_eq!(game.multiplier, before_finish * 2);
            let sign = if winner == 0 { 1 } else { -1 };
            let stake = 3 * game.multiplier as i32;
            assert_eq!(
                result.scores[0],
                sign * stake * i32::from(mode.players() - 1)
            );
            assert!(
                result.scores[1..]
                    .iter()
                    .all(|&score| score == -sign * stake)
            );
            let settled = result.clone();
            assert_eq!(game.act(winner, Action::Pass), Err(GameError::WrongPhase));
            assert_eq!(game.result.as_ref(), Some(&settled));
        }
    }
}
