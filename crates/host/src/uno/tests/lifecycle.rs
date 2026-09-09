use super::*;

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
