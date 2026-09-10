use super::*;

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
        52300,
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
