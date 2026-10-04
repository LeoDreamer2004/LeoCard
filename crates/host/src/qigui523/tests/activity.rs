use super::*;
use leocard_protocol::{ClientCommand, GameKind, PlayerId, PlayerInteractionKind, ServerEvent};
use leocard_qigui523::build_deck;

#[test]
fn game_start_announces_one_authoritative_fact_and_rejected_or_replayed_requests_do_not() {
    let mut session = QiGui523Session::new(ROOM, 52300, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    let rejected = send_next(&mut session, HOST, ClientCommand::StartGame);
    assert!(rejection(&rejected).is_some());
    assert!(
        !rejected
            .iter()
            .any(|delivery| matches!(delivery.message.event, ServerEvent::GameStarted { .. }))
    );
    for connection in [HOST, SECOND, THIRD] {
        send_next(
            &mut session,
            connection,
            ClientCommand::SetReady { ready: true },
        );
    }
    let started = send_next(&mut session, HOST, ClientCommand::StartGame);
    let facts = started
        .iter()
        .filter(|delivery| matches!(delivery.message.event, ServerEvent::GameStarted { .. }))
        .collect::<Vec<_>>();
    assert_eq!(facts.len(), 3);
    let match_id = session.match_id.unwrap();
    for delivery in &facts {
        assert_eq!(
            delivery.message.event,
            ServerEvent::GameStarted {
                host: PlayerId(0),
                game: GameKind::QiGui523,
                match_id
            }
        );
        let context = delivery.message.game_context.unwrap();
        assert_eq!(context.match_id, match_id);
        assert_eq!(context.hand_index, None);
        assert!(
            started
                .iter()
                .filter_map(|delivery| delivery.message.game_context)
                .all(|earlier| earlier.sequence <= context.sequence)
        );
    }
    let last_request = session.last_requests[&HOST].0;
    let replay = session.handle(HOST, message(last_request, ClientCommand::StartGame));
    assert!(
        !replay
            .iter()
            .any(|delivery| matches!(delivery.message.event, ServerEvent::GameStarted { .. }))
    );
    for command in [ClientCommand::StartGame, ClientCommand::RequestSnapshot] {
        let deliveries = send_next(&mut session, HOST, command);
        assert!(
            !deliveries
                .iter()
                .any(|delivery| matches!(delivery.message.event, ServerEvent::GameStarted { .. }))
        );
    }
}

#[test]
fn accepted_interactions_share_host_timing_and_accounting_but_replays_do_not() {
    let mut session = QiGui523Session::new(ROOM, 52300, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);
    let command = ClientCommand::Interact {
        target: PlayerId(1),
        kind: PlayerInteractionKind::Wine,
    };
    let deliveries = send_next(&mut session, HOST, command.clone());
    let events = deliveries
        .iter()
        .filter_map(|delivery| match delivery.message.event {
            ServerEvent::PlayerInteraction(event) => Some(event),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(events.len(), 3);
    assert!(events.iter().all(|event| *event == events[0]));
    let request = session.last_requests[&HOST].0;
    session.handle(HOST, message(request, command));
    let stats = session.players[1]
        .game_profiles
        .interactions
        .as_ref()
        .unwrap();
    assert_eq!(stats.flowers_received, 10);
    let later = send_next(
        &mut session,
        HOST,
        ClientCommand::Interact {
            target: PlayerId(1),
            kind: PlayerInteractionKind::Shoe,
        },
    );
    let later = later
        .iter()
        .find_map(|delivery| match delivery.message.event {
            ServerEvent::PlayerInteraction(event) => Some(event),
            _ => None,
        })
        .unwrap();
    assert!(later.elapsed_millis >= events[0].elapsed_millis);
    assert_eq!(
        session.players[1]
            .game_profiles
            .interactions
            .as_ref()
            .unwrap()
            .eggs_received,
        10
    );
}
