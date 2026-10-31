use super::*;
use crate::ClientPhaseRef;

#[tokio::test]
async fn being_kicked_ends_the_connection_without_reconnecting() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap().to_string();
    let (_commands, receiver) = tokio::sync::mpsc::channel(8);
    let events: EventQueue = Arc::new(Mutex::new(Default::default()));
    let worker_events = events.clone();
    let worker = tokio::spawn(async move {
        run_network(
            NetworkLaunch::Join { address },
            receiver,
            &worker_events,
            ClientMessage::new(NETWORK_ROOM_ID, RequestId(1), ClientCommand::Ping),
        )
        .await
    });
    let (mut stream, _) = listener.accept().await.unwrap();
    confirm_join(&mut stream).await;
    write_message(&mut stream, &message(ServerEvent::KickedFromRoom))
        .await
        .unwrap();
    drop(stream);
    tokio::time::timeout(Duration::from_secs(1), worker)
        .await
        .expect("a kick is terminal, rather than a reconnectable transport error")
        .unwrap()
        .unwrap();
    let events = events.lock().unwrap();
    assert!(events.iter().any(|event| matches!(event, NetworkEvent::Message(message) if matches!(message.event, ServerEvent::KickedFromRoom))));
    assert!(!events.iter().any(|event| matches!(
        event,
        NetworkEvent::Reconnecting(_) | NetworkEvent::Failed(_)
    )));
}

#[test]
fn kick_announcements_and_the_kicked_clients_exit_have_distinct_state() {
    let mut remaining = ClientModel::new(NETWORK_ROOM_ID);
    remaining.apply(message(ServerEvent::Joined { you: PlayerId(0) }));
    remaining.apply(snapshot());
    remaining.apply(message(ServerEvent::PlayerKicked {
        name: "访客".to_owned(),
    }));
    assert_eq!(remaining.last_notice(), Some("访客已被房主踢出"));
    assert!(matches!(remaining.phase(), ClientPhaseRef::Lobby(_)));
    assert!(!remaining.kicked_from_room());

    let mut kicked = remaining;
    kicked.apply(message(ServerEvent::KickedFromRoom));
    assert!(kicked.kicked_from_room());
    assert!(kicked.left_room());
    assert!(!kicked.room_closed());
    assert!(matches!(kicked.phase(), ClientPhaseRef::Closed));
}
