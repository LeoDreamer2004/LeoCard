use ed25519_dalek::{Signer, SigningKey};
use leocard_game_common::PlayerId;
use leocard_host::{ConnectionId, Delivery, GameSetup, HostSession};
use leocard_protocol::{
    ClientCommand, ClientMessage, GameKind, GameRules, JoinRequest, PlayerGameProfiles, ProfileId,
    ReconnectToken, RequestId, RoomId, SeatId, ServerEvent,
};

#[test]
fn fresh_hosts_receive_a_valid_seat_and_can_start_every_game_without_selecting_again() {
    let room = RoomId(88);
    let key = SigningKey::from_bytes(&[1; 32]);
    let mut join = JoinRequest {
        name: "房主".to_owned(),
        reconnect_token: ReconnectToken(1),
        profile_id: ProfileId(key.verifying_key().to_bytes()),
        reference_points: 0,
        completed_games: 0,
        game_profiles: PlayerGameProfiles::default(),
        identity_signature: Vec::new(),
    };
    join.identity_signature = key.sign(&join.identity_payload(room)).to_bytes().to_vec();
    for (rules, seats) in [
        (GameRules::QiGui523(Default::default()), 6),
        (GameRules::Texas(Default::default()), 8),
        (GameRules::Shengji(Default::default()), 4),
        (GameRules::Uno(Default::default()), 8),
        (GameRules::Mahjong(Default::default()), 4),
        (GameRules::McGuess(Default::default()), 8),
        (GameRules::Blackjack(Default::default()), 6),
        (GameRules::Doudizhu(Default::default()), 3),
    ] {
        let mut session = HostSession::new(room, GameSetup::shuffled(52300, rules)).unwrap();
        let mut request = 0;
        let mut send = |command| {
            request += 1;
            let deliveries = session.handle(
                ConnectionId(1),
                ClientMessage::new(room, RequestId(request), command),
            );
            assert_accepted(&deliveries);
            deliveries
        };
        let joined = send(ClientCommand::join(join.clone()));
        let lobby = joined
            .iter()
            .find_map(|delivery| match &delivery.message.event {
                ServerEvent::LobbySnapshot(lobby) => Some(lobby),
                _ => None,
            })
            .unwrap();
        assert_eq!(lobby.host, Some(lobby.players[0].player.id));
        let mut host_info = lobby.players[0].player.clone();
        let seat = lobby.players[0].seat.expect("fresh host must be seated");
        assert!(seat.0 < seats && lobby.players[0].ready);
        for other in (0..seats).filter(|other| *other != seat.0) {
            send(ClientCommand::ConfigureBotSeat {
                seat: SeatId(other),
                occupied: true,
            });
        }
        let started = send(ClientCommand::StartGame);
        assert!(
            started
                .iter()
                .any(|delivery| matches!(delivery.message.event, ServerEvent::GameStarted { .. }))
        );
        let snapshot = started
            .iter()
            .find_map(|delivery| match &delivery.message.event {
                ServerEvent::GameSnapshot(snapshot) => Some(snapshot),
                _ => None,
            })
            .expect("every game sends its initial snapshot");
        // Starting a match may reorder and renumber the seated participants.
        host_info.id = snapshot.you();
        assert_eq!(snapshot.host(), host_info.id);
        assert_eq!(snapshot.players().len(), usize::from(seats));
        assert_eq!(snapshot.player(host_info.id), Some(&host_info));
        assert!(snapshot.player(PlayerId(u8::MAX)).is_none());
        match snapshot.toggle_auto_play_command() {
            Some(command) => assert_eq!(command.kind(), snapshot.kind()),
            None => assert_eq!(snapshot.kind(), GameKind::McGuess),
        }
    }
}

fn assert_accepted(deliveries: &[Delivery]) {
    for delivery in deliveries {
        assert!(
            !matches!(delivery.message.event, ServerEvent::Rejected { .. }),
            "{delivery:?}"
        );
    }
}
