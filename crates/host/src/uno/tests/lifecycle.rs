use super::super::*;
use super::*;
use crate::ConnectionId;
use leocard_game_common::PlayerId;
use leocard_protocol::{
    ClientCommand, GameCommand, GameViolation, PlayerInteractionKind, RejectReason, SeatId,
    ServerEvent, TexasCommand, UnoCommand,
};
#[cfg(test)]
use leocard_uno::build_deck;
use leocard_uno::{UnoCard, UnoColor, UnoRuleSet};
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
            PlayerId(1)
        );
        assert!(session.advance_time(Duration::from_millis(999)).is_empty());
        assert_eq!(
            session.game().unwrap().turn().unwrap().current_player,
            PlayerId(1)
        );
        let deliveries = session.advance_time(Duration::from_millis(1));
        let game = session.game().unwrap();
        if stacking {
            assert!(deliveries.is_empty());
            assert_eq!(game.turn().unwrap().pending_skip, 1);
            assert_eq!(game.turn().unwrap().current_player, PlayerId(1));
        } else {
            assert_eq!(game.turn().unwrap().current_player, PlayerId(2));
            assert_eq!(game.turn().unwrap().pending_skip, 0);
            assert_eq!(
                game.player(PlayerId(1)).unwrap().hand().len(),
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

#[test]
fn wrong_game_command_is_rejected() {
    let mut session = UnoSession::new(ROOM, 52300, UnoRuleSet::default(), build_deck()).unwrap();
    session.handle(HOST, message(1, join_command("甲", 1)));
    let deliveries = session.handle(
        HOST,
        message(
            2,
            ClientCommand::Game(GameCommand::Texas(TexasCommand::SetAutoPlay {
                enabled: true,
            })),
        ),
    );
    assert!(matches!(
        deliveries[0].message.event,
        ServerEvent::Rejected {
            reason: RejectReason::Game(GameViolation::WrongGame)
        }
    ));
}

#[test]
fn lobby_interactions_broadcast_before_starting_a_game_and_reject_invalid_targets() {
    let mut session = UnoSession::new(ROOM, 52300, UnoRuleSet::default(), build_deck()).unwrap();
    session.handle(HOST, message(1, join_command("甲", 1)));
    session.handle(ConnectionId(2), message(1, join_command("乙", 2)));
    let deliveries = session.handle(
        HOST,
        message(
            2,
            ClientCommand::Interact {
                target: PlayerId(1),
                kind: PlayerInteractionKind::Flower,
            },
        ),
    );
    let interactions: Vec<_> = deliveries
        .iter()
        .filter_map(|delivery| {
            if let ServerEvent::PlayerInteraction(interaction) = &delivery.message.event {
                assert!(delivery.message.game_context.is_none());
                Some(interaction)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(interactions.len(), 2);
    assert!(interactions.iter().all(|interaction| {
        interaction.source == PlayerId(0)
            && interaction.target == PlayerId(1)
            && interaction.kind == PlayerInteractionKind::Flower
    }));
    assert!(session.game().is_none());
    for (request, target) in [(3, PlayerId(0)), (4, PlayerId(99))] {
        let rejected = session.handle(
            HOST,
            message(
                request,
                ClientCommand::Interact {
                    target,
                    kind: PlayerInteractionKind::Flower,
                },
            ),
        );
        assert!(
            rejected
                .iter()
                .any(|delivery| matches!(delivery.message.event, ServerEvent::Rejected { .. }))
        );
    }
}
