use super::*;
use ed25519_dalek::{Signer, SigningKey};
use leocard_protocol::{
    ClientCommand, ClientMessage, GameCommand, GameKind, GameSnapshot, GameViolation, PlayerId,
    RejectReason, RequestId, RoomId, ServerEvent, UnoCommand, UnoEvent, UnoProfileStats,
    UnoSnapshot, UnoViolation,
};
use leocard_protocol::{
    JoinRequest, PlayerGameProfiles, ProfileId, ReconnectToken, SeatId, TexasHoldemCommand,
    join_identity_payload,
};
use leocard_uno::{
    ActionOutcome, GameState, Phase, PlayedEffect, UnoCard, UnoChallengeResult, UnoColor,
    UnoPlayerId, UnoRuleSet, build_deck_for_rules,
};
use leocard_uno::{FlipRuleSet, Mode, UnoFace};
#[cfg(test)]
use leocard_uno::{build_deck, build_no_mercy_deck};

const ROOM: RoomId = RoomId(108);
const HOST: ConnectionId = ConnectionId(1);

fn message(request: u64, command: ClientCommand) -> ClientMessage {
    ClientMessage::new(ROOM, RequestId(request), command)
}

fn join_command(name: &str, token: u64) -> ClientCommand {
    let mut secret = [0; 32];
    secret[..8].copy_from_slice(&token.to_be_bytes());
    secret[8] = 7;
    let key = SigningKey::from_bytes(&secret);
    let reconnect_token = ReconnectToken(token);
    let game_profiles = PlayerGameProfiles::default();
    let payload = join_identity_payload(ROOM, reconnect_token, name, 0, 0, &game_profiles);
    ClientCommand::join(JoinRequest {
        name: name.to_owned(),
        reconnect_token,
        profile_id: ProfileId(key.verifying_key().to_bytes()),
        reference_points: 0,
        completed_games: 0,
        game_profiles,
        identity_signature: key.sign(&payload).to_bytes().to_vec(),
    })
}

#[test]
fn two_players_can_start_and_receive_private_hands() {
    let mut session = UnoSession::new(ROOM, UnoRuleSet::default(), build_deck()).unwrap();
    session.handle(HOST, message(1, join_command("甲", 1)));
    let second = ConnectionId(2);
    session.handle(second, message(1, join_command("乙", 2)));
    session.handle(second, message(2, ClientCommand::SetReady { ready: true }));
    let deliveries = session.handle(HOST, message(2, ClientCommand::StartGame));
    let snapshots = deliveries
        .iter()
        .filter_map(|delivery| match &delivery.message.event {
            ServerEvent::GameSnapshot(GameSnapshot::Uno(snapshot)) => Some(snapshot),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(snapshots.len(), 2);
    assert!(
        snapshots
            .iter()
            .all(|snapshot| snapshot.your_hand.len() == 7)
    );
    assert!(snapshots.iter().all(|snapshot| snapshot.players.len() == 2));
}

#[test]
fn flip_snapshots_hide_your_inactive_faces_but_show_opponents_backs() {
    let rules = UnoRuleSet {
        mode: Mode::Flip,
        ..UnoRuleSet::default()
    };
    let mut session = UnoSession::new(ROOM, rules, build_deck_for_rules(rules)).unwrap();
    session.handle(HOST, message(1, join_command("甲", 1)));
    let second = ConnectionId(2);
    session.handle(second, message(1, join_command("乙", 2)));
    session.handle(second, message(2, ClientCommand::SetReady { ready: true }));
    let deliveries = session.handle(HOST, message(2, ClientCommand::StartGame));

    for delivery in deliveries {
        let ServerEvent::GameSnapshot(GameSnapshot::Uno(snapshot)) = delivery.message.event else {
            continue;
        };
        let own = snapshot
            .players
            .iter()
            .find(|player| player.id == snapshot.you)
            .unwrap();
        assert!(own.inactive_hand.is_empty());
        assert!(
            snapshot
                .your_hand
                .iter()
                .all(|card| card.opposite().is_none())
        );
        assert!(
            snapshot
                .players
                .iter()
                .filter(|player| player.id != snapshot.you)
                .all(|player| player.inactive_hand.len() == player.hand_len as usize)
        );
        assert_eq!(snapshot.draw_pile_inactive_cards.len(), 6);
    }
}

#[test]
fn no_mercy_draw_until_playable_reveals_authoritative_cards_one_at_a_time() {
    fn place(deck: &mut [UnoCard], index: usize, card: UnoCard) {
        let current = deck
            .iter()
            .position(|candidate| *candidate == card)
            .expect("No Mercy deck contains the requested card");
        deck.swap(index, current);
    }

    let rules = UnoRuleSet {
        mode: Mode::NoMercy,
        ..UnoRuleSet::default()
    };
    let mut deck = build_no_mercy_deck();
    for (index, value) in [0, 1, 2, 3, 4, 6, 7].into_iter().enumerate() {
        place(
            &mut deck,
            index * 2,
            UnoCard::number(UnoColor::Blue, value, 0),
        );
    }
    let top = UnoCard::number(UnoColor::Red, 5, 0);
    let miss = UnoCard::number(UnoColor::Green, 8, 0);
    let playable = UnoCard::number(UnoColor::Red, 3, 0);
    place(&mut deck, 14, top);
    place(&mut deck, 15, miss);
    place(&mut deck, 16, playable);

    let mut session = UnoSession::new(ROOM, rules, deck).unwrap();
    let second = ConnectionId(2);
    session.handle(HOST, message(1, join_command("甲", 1)));
    session.handle(second, message(1, join_command("乙", 2)));
    for (index, player) in session.room.players.iter_mut().enumerate() {
        player.seat = Some(SeatId(index as u8));
    }
    session.handle(second, message(3, ClientCommand::SetReady { ready: true }));
    session.handle(HOST, message(3, ClientCommand::StartGame));

    let deliveries = session.handle(
        HOST,
        message(
            4,
            ClientCommand::Game(GameCommand::Uno(UnoCommand::DrawCard)),
        ),
    );
    let snapshot = deliveries
        .iter()
        .find_map(|delivery| match &delivery.message.event {
            ServerEvent::GameSnapshot(GameSnapshot::Uno(snapshot))
                if delivery.recipient == HOST =>
            {
                Some(snapshot)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(snapshot.your_hand.len(), 7);
    assert_eq!(snapshot.current_player, None);
    assert!(session.advance_time(Duration::from_millis(779)).is_empty());

    let first = session.advance_time(Duration::from_millis(1));
    let first = first
        .iter()
        .find_map(|delivery| match &delivery.message.event {
            ServerEvent::GameSnapshot(GameSnapshot::Uno(snapshot))
                if delivery.recipient == HOST =>
            {
                Some(snapshot)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(first.your_hand.len(), 8);
    assert!(first.your_hand.contains(&miss));
    assert_eq!(first.current_player, None);

    let second = session.advance_time(DRAW_REVEAL_INTERVAL);
    let second = second
        .iter()
        .find_map(|delivery| match &delivery.message.event {
            ServerEvent::GameSnapshot(GameSnapshot::Uno(snapshot))
                if delivery.recipient == HOST =>
            {
                Some(snapshot)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(second.your_hand.len(), 9);
    assert!(second.your_hand.contains(&playable));
    assert_eq!(second.current_player, Some(PlayerId(0)));
    assert_eq!(second.your_drawn_card, Some(playable));
}

#[test]
fn flip_draw_events_expose_the_matching_card_backs_in_order() {
    let cards = build_deck_for_rules(UnoRuleSet {
        mode: Mode::Flip,
        ..UnoRuleSet::default()
    })
    .into_iter()
    .take(3)
    .collect::<Vec<_>>();
    let expected = cards
        .iter()
        .filter_map(|card| card.opposite_public_face())
        .collect::<Vec<_>>();

    let outcome = ActionOutcome::ColorRouletteResolved {
        player: UnoPlayerId(0),
        color: UnoColor::Pink,
        cards: cards.clone(),
        next_player: UnoPlayerId(1),
    };
    let events = events_for_outcome(&outcome, &[]);
    assert_eq!(
        events,
        vec![UnoEvent::ColorRouletteResolved {
            player: PlayerId(0),
            color: UnoColor::Pink,
            count: 3,
            card_backs: expected,
        }]
    );

    let game = GameState::new_with_deck(
        UnoRuleSet {
            mode: Mode::Flip,
            ..UnoRuleSet::default()
        },
        2,
        build_deck_for_rules(UnoRuleSet {
            mode: Mode::Flip,
            ..UnoRuleSet::default()
        }),
    )
    .unwrap();
    let reveal = pending_draw_reveal_for_outcome(&outcome, &game).unwrap();
    assert_eq!(reveal.cards, cards);
    assert_eq!(reveal.revealed, 0);
    assert_eq!(reveal.remaining, COLOR_ROULETTE_REVEAL_START_DELAY);
}

#[test]
fn flip_color_draw_reveals_cards_and_draw_pile_backs_one_at_a_time() {
    let rules = UnoRuleSet {
        mode: Mode::Flip,
        ..UnoRuleSet::default()
    };
    let mut session = UnoSession::new(ROOM, rules, build_deck_for_rules(rules)).unwrap();
    let second = ConnectionId(2);
    session.handle(HOST, message(1, join_command("甲", 1)));
    session.handle(second, message(1, join_command("乙", 2)));
    session.handle(second, message(2, ClientCommand::SetReady { ready: true }));
    session.handle(HOST, message(2, ClientCommand::StartGame));

    let cards = session
        .game
        .as_ref()
        .unwrap()
        .player(UnoPlayerId(0))
        .unwrap()
        .hand()
        .iter()
        .copied()
        .take(2)
        .collect::<Vec<_>>();
    session.pending_draw_reveal = Some(PendingDrawReveal {
        player: PlayerId(0),
        cards: cards.clone(),
        revealed: 0,
        remaining: COLOR_ROULETTE_REVEAL_START_DELAY,
    });

    let initial = session.game_snapshot(PlayerId(1));
    assert_eq!(
        initial.draw_pile_inactive_cards.first().copied(),
        cards[0].opposite_public_face()
    );
    assert_eq!(
        initial.players[0].hand_len,
        session
            .game
            .as_ref()
            .unwrap()
            .player(UnoPlayerId(0))
            .unwrap()
            .hand()
            .len() as u8
            - 2
    );

    session.advance_time(COLOR_ROULETTE_REVEAL_START_DELAY);
    let after_first = session.game_snapshot(PlayerId(1));
    assert_eq!(
        after_first.draw_pile_inactive_cards.first().copied(),
        cards[1].opposite_public_face()
    );
    assert_eq!(
        after_first.players[0].hand_len,
        initial.players[0].hand_len + 1
    );

    session.advance_time(DRAW_REVEAL_INTERVAL);
    assert!(session.pending_draw_reveal.is_none());
}

#[test]
fn uno_call_and_report_remain_reactive_during_incremental_draws() {
    let rules = UnoRuleSet {
        mode: Mode::Flip,
        ..UnoRuleSet::default()
    };
    let mut session = UnoSession::new(ROOM, rules, build_deck_for_rules(rules)).unwrap();
    let second = ConnectionId(2);
    session.handle(HOST, message(1, join_command("甲", 1)));
    session.handle(second, message(1, join_command("乙", 2)));
    session.handle(second, message(2, ClientCommand::SetReady { ready: true }));
    session.handle(HOST, message(2, ClientCommand::StartGame));

    let cards = session
        .game
        .as_ref()
        .unwrap()
        .player(UnoPlayerId(0))
        .unwrap()
        .hand()[..2]
        .to_vec();
    session.pending_draw_reveal = Some(PendingDrawReveal {
        player: PlayerId(0),
        cards,
        revealed: 0,
        remaining: COLOR_ROULETTE_REVEAL_START_DELAY,
    });

    let call = session.handle(
        HOST,
        message(
            3,
            ClientCommand::Game(GameCommand::Uno(UnoCommand::CallUno)),
        ),
    );
    assert!(call.iter().any(|delivery| {
        matches!(
            delivery.message.event,
            ServerEvent::Rejected {
                reason: RejectReason::Game(GameViolation::Uno(UnoViolation::CannotCallUno))
            }
        )
    }));
    assert!(session.pending_draw_reveal.is_some());

    let reporter = session.room.player_id(second).unwrap();
    let target = session
        .room
        .players
        .iter()
        .find(|player| player.id != reporter)
        .unwrap()
        .id;
    let report = session.handle(
        second,
        message(
            3,
            ClientCommand::Game(GameCommand::Uno(UnoCommand::ReportUno { target })),
        ),
    );
    assert!(report.iter().any(|delivery| {
        matches!(
            delivery.message.event,
            ServerEvent::Rejected {
                reason: RejectReason::Game(GameViolation::Uno(UnoViolation::PlayerNotReportable))
            }
        )
    }));
    assert!(session.pending_draw_reveal.is_some());
}

#[test]
fn rematch_keeps_auto_play_after_clearing_ready_state() {
    let mut session = UnoSession::new(ROOM, UnoRuleSet::default(), build_deck()).unwrap();
    session.handle(HOST, message(1, join_command("甲", 1)));
    let second = ConnectionId(2);
    session.handle(second, message(1, join_command("乙", 2)));
    session.handle(second, message(2, ClientCommand::SetReady { ready: true }));
    session.handle(HOST, message(2, ClientCommand::StartGame));
    let participant = session
        .room
        .players
        .iter_mut()
        .find(|player| player.connection == second)
        .unwrap();
    participant.auto_play = true;

    session.room.prepare_rematch();
    let participant = session
        .room
        .players
        .iter()
        .find(|player| player.connection == second)
        .unwrap();
    assert!(participant.ready);
    assert!(participant.auto_play);

    session.start_next_game(None);
    let participant = session
        .room
        .players
        .iter()
        .find(|player| player.connection == second)
        .unwrap();
    assert!(!participant.ready);
    assert!(participant.auto_play);
}

#[cfg(feature = "developer")]
#[test]
fn changing_rules_keeps_developer_bots_ready() {
    let mut session = UnoSession::new(ROOM, UnoRuleSet::default(), build_deck()).unwrap();
    session.handle(HOST, message(1, join_command("甲", 1)));
    session.handle(
        HOST,
        message(
            2,
            ClientCommand::ConfigureBotSeat {
                seat: SeatId(1),
                occupied: true,
            },
        ),
    );

    session.handle(
        HOST,
        message(
            3,
            ClientCommand::Game(GameCommand::Uno(UnoCommand::UpdateRules {
                rules: UnoRuleSet {
                    uno_callout: false,
                    ..UnoRuleSet::default()
                },
            })),
        ),
    );

    assert!(
        session
            .room
            .players
            .iter()
            .filter(|player| player.is_bot)
            .all(|player| player.ready)
    );
    let deliveries = session.handle(HOST, message(4, ClientCommand::StartGame));
    assert!(deliveries.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::GameSnapshot(GameSnapshot::Uno(_))
    )));
}

#[test]
fn wrong_game_command_is_rejected_with_uno_as_expected_kind() {
    let mut session = UnoSession::new(ROOM, UnoRuleSet::default(), build_deck()).unwrap();
    session.handle(HOST, message(1, join_command("甲", 1)));
    let deliveries = session.handle(
        HOST,
        message(
            2,
            ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::SetAutoPlay {
                enabled: true,
            })),
        ),
    );
    assert!(matches!(
        deliveries[0].message.event,
        ServerEvent::Rejected {
            reason: RejectReason::Game(GameViolation::WrongGame {
                expected: GameKind::Uno,
                received: GameKind::TexasHoldem,
            })
        }
    ));
}

fn jump_in_session() -> (UnoSession, ConnectionId, ConnectionId, UnoCard, UnoCard) {
    let first = UnoCard::number(UnoColor::Red, 7, 0);
    let matching = UnoCard::number(UnoColor::Red, 7, 1);
    let start = UnoCard::number(UnoColor::Red, 5, 0);
    let mut deck = build_deck();
    for (position, required) in [(0, first), (2, matching), (21, start)] {
        let current = deck
            .iter()
            .position(|candidate| *candidate == required)
            .unwrap();
        deck.swap(position, current);
    }
    let mut session = UnoSession::new(
        ROOM,
        UnoRuleSet {
            action_stacking: true,
            jump_in: true,
            ..UnoRuleSet::default()
        },
        deck,
    )
    .unwrap();
    let second = ConnectionId(2);
    let third = ConnectionId(3);
    session.handle(HOST, message(1, join_command("甲", 1)));
    session.handle(second, message(1, join_command("乙", 2)));
    session.handle(third, message(1, join_command("丙", 3)));
    for (index, player) in session.room.players.iter_mut().enumerate() {
        player.seat = Some(SeatId(index as u8));
    }
    session.handle(second, message(2, ClientCommand::SetReady { ready: true }));
    session.handle(third, message(2, ClientCommand::SetReady { ready: true }));
    session.handle(HOST, message(2, ClientCommand::StartGame));
    (session, second, third, first, matching)
}

fn uno_snapshot_for(deliveries: &[Delivery], recipient: ConnectionId) -> &UnoSnapshot {
    deliveries
        .iter()
        .find_map(|delivery| {
            (delivery.recipient == recipient)
                .then_some(&delivery.message.event)
                .and_then(|event| match event {
                    ServerEvent::GameSnapshot(GameSnapshot::Uno(snapshot)) => Some(snapshot),
                    _ => None,
                })
        })
        .expect("recipient should receive an UNO snapshot")
}

#[test]
fn jump_in_candidate_is_private_and_successful_command_moves_play_to_that_player() {
    let (mut session, second, third, first, matching) = jump_in_session();
    let deliveries = session.handle(
        HOST,
        message(
            3,
            ClientCommand::Game(GameCommand::Uno(UnoCommand::PlayCard {
                card: first,
                chosen_color: None,
            })),
        ),
    );
    assert_eq!(
        uno_snapshot_for(&deliveries, second).your_jump_in_card,
        None
    );
    assert_eq!(
        uno_snapshot_for(&deliveries, third).your_jump_in_card,
        Some(matching)
    );

    let deliveries = session.handle(
        third,
        message(
            3,
            ClientCommand::Game(GameCommand::Uno(UnoCommand::JumpIn { card: matching })),
        ),
    );
    let snapshot = uno_snapshot_for(&deliveries, third);
    assert_eq!(snapshot.discard_top, matching);
    assert_eq!(snapshot.current_player, Some(PlayerId(0)));
    assert_eq!(snapshot.your_hand.len(), 6);
    assert_eq!(session.match_profile_stats[2].jump_in_opportunities, 1);
    assert_eq!(session.match_profile_stats[2].successful_jump_ins, 1);
}

#[test]
fn any_successful_next_player_action_closes_server_jump_in_window() {
    let (mut session, second, third, first, matching) = jump_in_session();
    session.handle(
        HOST,
        message(
            3,
            ClientCommand::Game(GameCommand::Uno(UnoCommand::PlayCard {
                card: first,
                chosen_color: None,
            })),
        ),
    );
    session.handle(
        second,
        message(
            3,
            ClientCommand::Game(GameCommand::Uno(UnoCommand::DrawCard)),
        ),
    );
    let deliveries = session.handle(
        third,
        message(
            3,
            ClientCommand::Game(GameCommand::Uno(UnoCommand::JumpIn { card: matching })),
        ),
    );
    assert!(deliveries.iter().any(|delivery| {
        delivery.recipient == third
            && matches!(
                delivery.message.event,
                ServerEvent::Rejected {
                    reason: RejectReason::Game(GameViolation::Uno(UnoViolation::CannotJumpIn))
                }
            )
    }));
    assert_eq!(session.match_profile_stats[2].jump_in_opportunities, 1);
    assert_eq!(session.match_profile_stats[2].successful_jump_ins, 0);
}

#[test]
fn non_play_actions_do_not_count_the_same_jump_in_window_twice() {
    let (mut session, _, third, first, _) = jump_in_session();
    session.handle(
        HOST,
        message(
            3,
            ClientCommand::Game(GameCommand::Uno(UnoCommand::PlayCard {
                card: first,
                chosen_color: None,
            })),
        ),
    );
    assert_eq!(session.match_profile_stats[2].jump_in_opportunities, 1);

    session.perform_action(third, RequestId(4), Vec::new(), false, |_, player| {
        Ok(ActionOutcome::UnoCalled { player })
    });

    assert_eq!(session.match_profile_stats[2].jump_in_opportunities, 1);
}

#[test]
fn public_flip_card_command_resolves_to_the_private_physical_card() {
    let rules = UnoRuleSet {
        mode: Mode::Flip,
        ..UnoRuleSet::default()
    };
    let game = GameState::new_with_deck(rules, 2, build_deck_for_rules(rules)).unwrap();
    let actual = game.player(UnoPlayerId(0)).unwrap().hand()[0];

    assert_eq!(
        resolve_public_hand_card(&game, UnoPlayerId(0), actual.public_face()),
        Ok(actual)
    );
}

#[test]
fn profile_events_record_uno_penalties_and_challenge_results() {
    let mut session = UnoSession::new(ROOM, UnoRuleSet::default(), build_deck()).unwrap();
    session.match_profile_stats = vec![UnoProfileStats::default(); 3];
    session.record_profile_outcome(&ActionOutcome::UnoCalled {
        player: UnoPlayerId(0),
    });
    session.record_profile_outcome(&ActionOutcome::UnoReported {
        reporter: UnoPlayerId(1),
        target: UnoPlayerId(0),
        cards: vec![
            UnoCard::number(UnoColor::Blue, 1, 0),
            UnoCard::number(UnoColor::Blue, 2, 0),
        ],
    });
    session.record_profile_outcome(&ActionOutcome::ChallengeResolved {
        challenger: UnoPlayerId(1),
        offender: UnoPlayerId(2),
        result: UnoChallengeResult::Successful,
        penalized: UnoPlayerId(2),
        cards: vec![UnoCard::number(UnoColor::Green, 3, 0); 4],
        next_player: UnoPlayerId(1),
    });
    session.record_profile_outcome(&ActionOutcome::ChallengeResolved {
        challenger: UnoPlayerId(0),
        offender: UnoPlayerId(2),
        result: UnoChallengeResult::Failed,
        penalized: UnoPlayerId(0),
        cards: vec![UnoCard::number(UnoColor::Yellow, 4, 0); 6],
        next_player: UnoPlayerId(1),
    });

    assert_eq!(session.match_profile_stats[0].uno_calls, 1);
    assert_eq!(session.match_profile_stats[0].uno_penalties, 1);
    assert_eq!(session.match_profile_stats[0].challenges, 1);
    assert_eq!(session.match_profile_stats[0].successful_challenges, 0);
    assert_eq!(session.match_profile_stats[0].max_penalty_cards, 6);
    assert_eq!(session.match_profile_stats[1].challenges, 1);
    assert_eq!(session.match_profile_stats[1].successful_challenges, 1);
    assert_eq!(session.match_profile_stats[2].challenges_received, 2);
    assert_eq!(
        session.match_profile_stats[2].successful_challenges_received,
        1
    );
    assert_eq!(session.match_profile_stats[2].max_penalty_cards, 4);
}

#[test]
fn finished_game_merges_uno_profile_statistics_once() {
    let mut session = UnoSession::new(ROOM, UnoRuleSet::default(), build_deck()).unwrap();
    session.handle(HOST, message(1, join_command("甲", 1)));
    let second = ConnectionId(2);
    session.handle(second, message(1, join_command("乙", 2)));
    session.handle(second, message(2, ClientCommand::SetReady { ready: true }));
    session.handle(HOST, message(2, ClientCommand::StartGame));
    assert_eq!(session.match_profile_stats[0].max_hand_cards, 7);
    session.match_profile_stats[0].max_hand_cards = 40;
    session.match_profile_stats[0].max_penalty_cards = 12;
    session.match_profile_stats[0].challenges = 3;
    session.match_profile_stats[0].successful_challenges = 2;

    for _ in 0..5_000 {
        if session
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.phase(), Phase::Finished(_)))
        {
            break;
        }
        session
            .play_automatic_action()
            .expect("an automatic UNO action should remain available");
    }
    let result = match session.game.as_ref().unwrap().phase() {
        Phase::Finished(result) => result.clone(),
        Phase::Playing => panic!("automatic play did not finish the game"),
    };

    session.apply_finished_reference_points();
    session.apply_finished_reference_points();

    for (index, participant) in session.room.players.iter().enumerate() {
        let stats = participant.game_profiles.uno.as_ref().unwrap();
        assert_eq!(stats.completed_games, 1);
        assert_eq!(
            stats.total_reference_delta,
            i64::from(result.reference_deltas[index])
        );
        assert_eq!(
            stats.total_remaining_score,
            u64::from(result.hand_scores[index])
        );
        assert_eq!(stats.placement_counts.iter().sum::<u32>(), 1);
    }
    let first = session.room.players[0].game_profiles.uno.as_ref().unwrap();
    assert_eq!(first.max_hand_cards, 40);
    assert!(first.max_penalty_cards >= 12);
    assert_eq!(first.challenges, 3);
    assert_eq!(first.successful_challenges, 2);
}

#[test]
fn identical_pair_broadcasts_both_card_play_events_in_order() {
    let first = UnoCard::number(UnoColor::Red, 7, 0);
    let second = UnoCard::number(UnoColor::Red, 7, 1);
    let events = events_for_outcome(
        &ActionOutcome::Played {
            player: UnoPlayerId(0),
            card: second,
            next_player: UnoPlayerId(1),
            effect: None,
        },
        &[(first, None), (second, None)],
    );
    assert_eq!(
        events,
        vec![
            UnoEvent::CardPlayed {
                player: PlayerId(0),
                card: first,
                chosen_color: None,
                play_index: 0,
                play_count: 2,
            },
            UnoEvent::CardPlayed {
                player: PlayerId(0),
                card: second,
                chosen_color: None,
                play_index: 1,
                play_count: 2,
            },
        ]
    );
}

#[test]
fn stack_number_broadcasts_every_revealed_card_after_the_play() {
    let played = UnoCard::wild(UnoFace::WildStackNumber, 0);
    let skipped = UnoCard::action(UnoColor::Blue, UnoFace::Skip, 0);
    let number = UnoCard::number(UnoColor::Yellow, 6, 0);
    let events = events_for_outcome(
        &ActionOutcome::Played {
            player: UnoPlayerId(0),
            card: played,
            next_player: UnoPlayerId(1),
            effect: Some(PlayedEffect::StackNumberRevealed {
                cards: vec![skipped, number],
                value: 6,
            }),
        },
        &[(played, Some(UnoColor::Red))],
    );
    assert_eq!(
        events,
        vec![
            UnoEvent::CardPlayed {
                player: PlayerId(0),
                card: played,
                chosen_color: Some(UnoColor::Red),
                play_index: 0,
                play_count: 1,
            },
            UnoEvent::StackNumberRevealed {
                player: PlayerId(0),
                cards: vec![skipped, number],
                value: 6,
            },
        ]
    );
}

#[test]
fn random_flip_pairing_remains_valid_and_public_faces_keep_unique_ids() {
    let rules = UnoRuleSet {
        mode: Mode::Flip,
        flip: FlipRuleSet {
            random_pairing: true,
            ..FlipRuleSet::default()
        },
        ..UnoRuleSet::default()
    };
    let deck = shuffled_uno_deck(rules);
    validate_deck(&deck, rules).unwrap();
    assert_eq!(
        deck.iter()
            .map(|card| card.public_face())
            .collect::<HashSet<_>>()
            .len(),
        112
    );
    assert_eq!(
        deck.iter()
            .filter_map(|card| card.opposite_public_face())
            .collect::<HashSet<_>>()
            .len(),
        112
    );
}
