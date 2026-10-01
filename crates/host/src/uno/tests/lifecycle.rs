use super::super::*;
use super::*;
use crate::ConnectionId;
#[cfg(feature = "developer")]
use leocard_protocol::GameSnapshot;
use leocard_protocol::{
    ClientCommand, GameCommand, GameKind, GameViolation, PlayerId, RejectReason, ServerEvent,
    UnoCommand,
};
use leocard_protocol::{SeatId, TexasHoldemCommand};
#[cfg(test)]
use leocard_uno::build_deck;
use leocard_uno::{UnoCard, UnoColor, UnoPlayerId, UnoRuleSet};
use std::time::Duration;

#[test]
fn human_skip_is_automatic_only_without_stacking_and_keeps_its_penalty() {
    for (stacking, penalty) in [(false, false), (false, true), (true, false)] {
        let skip = UnoCard::action(UnoColor::Red, leocard_uno::UnoFace::Skip, 0);
        let start = UnoCard::number(UnoColor::Red, 5, 0);
        let mut deck = build_deck();
        for (position, required) in [(0, skip), (21, start)] {
            let current = deck.iter().position(|card| *card == required).unwrap();
            deck.swap(position, current);
        }
        let mut session = UnoSession::new(
            ROOM,
            52300,
            UnoRuleSet {
                action_stacking: stacking,
                skip_draw_penalty: penalty,
                ..UnoRuleSet::default()
            },
            deck,
        )
        .unwrap();
        for index in 0..3 {
            let connection = ConnectionId(index + 1);
            session.handle(connection, message(1, join_command("玩家", index + 1)));
            session.room.players[index as usize].seat = Some(SeatId(index as u8));
            if index > 0 {
                session.handle(
                    connection,
                    message(2, ClientCommand::SetReady { ready: true }),
                );
            }
        }
        session.handle(HOST, message(2, ClientCommand::StartGame));
        session.handle(
            HOST,
            message(
                3,
                ClientCommand::Game(GameCommand::Uno(UnoCommand::PlayCard {
                    card: skip,
                    chosen_color: None,
                })),
            ),
        );
        assert_eq!(
            session.game().unwrap().turn().unwrap().current_player,
            UnoPlayerId(1)
        );
        assert!(session.advance_time(Duration::from_millis(999)).is_empty());
        assert_eq!(
            session.game().unwrap().turn().unwrap().current_player,
            UnoPlayerId(1)
        );
        let deliveries = session.advance_time(Duration::from_millis(1));
        let game = session.game().unwrap();
        if stacking {
            assert!(deliveries.is_empty());
            assert_eq!(game.turn().unwrap().pending_skip, 1);
            assert_eq!(game.turn().unwrap().current_player, UnoPlayerId(1));
        } else {
            assert_eq!(game.turn().unwrap().current_player, UnoPlayerId(2));
            assert_eq!(game.turn().unwrap().pending_skip, 0);
            assert_eq!(
                game.player(UnoPlayerId(1)).unwrap().hand().len(),
                7 + usize::from(penalty)
            );
            assert!(deliveries.iter().any(|delivery| matches!(
                delivery.message.event,
                ServerEvent::GameEvent(leocard_protocol::GameEvent::Uno(leocard_protocol::UnoEvent::SkipResolved {
                    player: PlayerId(1), drew_card, ..
                })) if drew_card == penalty
            )));
            assert!(session.advance_time(Duration::from_secs(2)).is_empty());
        }
    }
}

#[test]
fn rematch_keeps_auto_play_after_clearing_ready_state() {
    let mut session = UnoSession::new(ROOM, 52300, UnoRuleSet::default(), build_deck()).unwrap();
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
    let mut session = UnoSession::new(ROOM, 52300, UnoRuleSet::default(), build_deck()).unwrap();
    session.handle(HOST, message(1, join_command("甲", 1)));
    session.handle(
        HOST,
        message(2, ClientCommand::SelectSeat { seat: SeatId(0) }),
    );
    session.handle(
        HOST,
        message(
            3,
            ClientCommand::ConfigureBotSeat {
                seat: SeatId(1),
                occupied: true,
            },
        ),
    );

    session.handle(
        HOST,
        message(
            4,
            ClientCommand::Game(GameCommand::Uno(UnoCommand::UpdateRules {
                rules: UnoRuleSet {
                    uno_callout: false,
                    ..UnoRuleSet::default()
                },
            })),
        ),
    );

    assert!(session.room.players.iter().any(|player| player.is_bot));
    assert!(
        session
            .room
            .players
            .iter()
            .filter(|player| player.is_bot)
            .all(|player| player.ready)
    );
    let deliveries = session.handle(HOST, message(5, ClientCommand::StartGame));
    assert!(deliveries.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::GameSnapshot(GameSnapshot::Uno(_))
    )));
}

#[test]
fn wrong_game_command_is_rejected_with_uno_as_expected_kind() {
    let mut session = UnoSession::new(ROOM, 52300, UnoRuleSet::default(), build_deck()).unwrap();
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
