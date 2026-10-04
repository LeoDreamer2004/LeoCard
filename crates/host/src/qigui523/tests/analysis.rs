use super::*;
use leocard_protocol::{
    ClientCommand, GameCommand, GameEvent, QiGui523Command, QiGui523Event, ServerEvent,
    ServerMessage, decode_frame, encode_frame,
};
use leocard_qigui523::{Phase, build_deck};

#[test]
fn private_live_reports_round_trip_and_failed_replayed_or_snapshot_requests_do_not_emit_them() {
    let mut session = QiGui523Session::new(ROOM, 52300, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);
    let game = session.game().unwrap();
    let actor = game.trick().unwrap().current_player();
    let connection = connection_for_player(&session, from_core_player(actor));
    let cards = vec![game.player(actor).unwrap().hand()[0]];
    let command = ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::PlayCards { cards }));
    let wrong = if connection == HOST { SECOND } else { HOST };
    let rejected = send_next(&mut session, wrong, command.clone());
    assert!(rejection(&rejected).is_some());
    assert_eq!(
        session.statistics.as_ref().unwrap().players()[actor.0].plays,
        0
    );
    let deliveries = send_next(&mut session, connection, command.clone());
    let mut reports = 0;
    for delivery in &deliveries {
        let ServerEvent::GameEvent(GameEvent::QiGui523(QiGui523Event::ActionAnalyzed {
            player,
            statistics,
        })) = &delivery.message.event
        else {
            continue;
        };
        reports += 1;
        assert_eq!(session.player_id(delivery.recipient), Some(*player));
        assert_eq!(usize::from(player.0), statistics.player.0);
        assert_eq!(
            statistics.progress.plays,
            u32::from(statistics.player == actor)
        );
        assert_eq!(
            delivery.message.game_context.unwrap().match_id,
            session.match_id.unwrap()
        );
        let decoded: ServerMessage =
            decode_frame(&encode_frame(&delivery.message).unwrap()).unwrap();
        assert_eq!(decoded, delivery.message);
    }
    assert_eq!(reports, 3);
    let request = session.last_requests[&connection].0;
    for deliveries in [
        session.handle(connection, message(request, command)),
        send_next(&mut session, connection, ClientCommand::RequestSnapshot),
    ] {
        assert!(!deliveries.iter().any(|delivery| matches!(
            delivery.message.event,
            ServerEvent::GameEvent(GameEvent::QiGui523(QiGui523Event::ActionAnalyzed { .. }))
        )));
    }
    assert_eq!(
        session.statistics.as_ref().unwrap().players()[actor.0].plays,
        1
    );
}

#[test]
fn automatic_actions_share_statistics_and_completed_profiles_settle_once() {
    let mut session = QiGui523Session::new(ROOM, 52300, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);
    for _ in 0..1000 {
        if matches!(session.game().unwrap().phase(), Phase::Finished(_)) {
            break;
        }
        let events = session.play_automatic_action();
        assert!(
            events
                .iter()
                .any(|event| matches!(event, QiGui523Event::ActionAnalyzed { .. }))
        );
        session.reset_timer_for_current_turn();
        session.bump_revision();
        session.broadcast_game_after_action(None, events);
    }
    assert!(matches!(
        session.game().unwrap().phase(),
        Phase::Finished(_)
    ));
    let progress = session.statistics.as_ref().unwrap().players();
    assert!(progress.iter().map(|p| p.plays).sum::<u32>() > 0);
    for (participant, progress) in session.players.iter().zip(progress) {
        let profile = participant.game_profiles.qigui523.as_ref().unwrap();
        assert_eq!(profile.completed_games, 1);
        assert_eq!(profile.straight_plays, progress.straight_plays);
        assert_eq!(profile.bomb_plays, progress.bomb_plays);
        assert_eq!(profile.heaven_bomb_plays, progress.heaven_bomb_plays);
    }
    session.broadcast_game_after_update(None);
    assert!(
        session
            .players
            .iter()
            .all(|p| p.game_profiles.qigui523.as_ref().unwrap().completed_games == 1)
    );
}
