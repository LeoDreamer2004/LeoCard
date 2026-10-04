use super::super::{UnoSession, preferred_color};
use super::*;
use crate::ConnectionId;
use leocard_protocol::{
    ClientCommand, GameCommand, GameEvent, PlayerId, ServerEvent, ServerMessage, UnoCommand,
    UnoEvent, decode_frame, encode_frame,
};
use leocard_uno::{Phase, UnoRuleSet, build_deck};

fn session() -> UnoSession {
    let mut session = UnoSession::new(ROOM, 52300, UnoRuleSet::default(), build_deck()).unwrap();
    session.handle(HOST, message(1, join_command("甲", 1)));
    let second = ConnectionId(2);
    session.handle(second, message(1, join_command("乙", 2)));
    session.handle(second, message(2, ClientCommand::SetReady { ready: true }));
    session.handle(HOST, message(2, ClientCommand::StartGame));
    session
}

#[test]
fn analysis_is_private_sequenced_and_only_emitted_for_accepted_actions() {
    let mut session = session();
    let rejected = session.handle(
        HOST,
        message(
            3,
            ClientCommand::Game(GameCommand::Uno(UnoCommand::CallUno)),
        ),
    );
    assert!(!rejected.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::GameEvent(GameEvent::Uno(UnoEvent::ActionAnalyzed { .. }))
    )));
    assert_eq!(statistics(&session, 0).uno_calls, 0);
    let game = session.game().unwrap();
    let player = game.turn().unwrap().current_player;
    let card = game
        .player(player)
        .unwrap()
        .hand()
        .iter()
        .copied()
        .find(|card| game.can_play(player, *card))
        .unwrap();
    let color = card.face().is_wild().then(|| preferred_color(game, player));
    let connection = session.room.players[player.0].connection;
    let request = session.room.last_requests[&connection].0 + 1;
    let command = ClientCommand::Game(GameCommand::Uno(UnoCommand::PlayCard {
        card,
        chosen_color: color,
    }));
    let deliveries = session.handle(connection, message(request, command.clone()));
    let mut found = false;
    for delivery in &deliveries {
        if let ServerEvent::GameEvent(GameEvent::Uno(UnoEvent::ActionAnalyzed { player, .. })) =
            &delivery.message.event
        {
            found = true;
            assert_eq!(session.room.player_id(delivery.recipient), Some(*player));
            assert!(delivery.message.game_context.is_some());
            let decoded: ServerMessage =
                decode_frame(&encode_frame(&delivery.message).unwrap()).unwrap();
            assert_eq!(decoded, delivery.message);
        }
    }
    assert!(found);
    let replay = session.handle(connection, message(request, command));
    assert!(!replay.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::GameEvent(GameEvent::Uno(UnoEvent::ActionAnalyzed { .. }))
    )));
}

#[test]
fn automatic_actions_emit_one_completion_per_player_and_snapshots_do_not_replay_it() {
    let mut session = session();
    let mut completions = [0; 2];
    for _ in 0..5_000 {
        if matches!(session.game().unwrap().phase(), Phase::Finished(_)) {
            break;
        }
        let (events, _) = session.play_automatic_action().unwrap();
        session.room.bump_revision();
        for delivery in session.broadcast_events(events) {
            if let ServerEvent::GameEvent(GameEvent::Uno(UnoEvent::ActionAnalyzed {
                player,
                statistics,
            })) = delivery.message.event
            {
                assert_eq!(session.room.player_id(delivery.recipient), Some(player));
                assert_eq!(player, PlayerId(statistics.player.0 as u8));
                assert!(delivery.message.game_context.is_some());
                if statistics.completed_game {
                    completions[usize::from(player.0)] += 1;
                }
            }
        }
    }
    assert_eq!(completions, [1, 1]);
    let snapshots = session.handle(HOST, message(3, ClientCommand::RequestSnapshot));
    assert!(!snapshots.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::GameEvent(GameEvent::Uno(UnoEvent::ActionAnalyzed { .. }))
    )));
}
