use super::*;
use leocard_guandan::GuandanAction;
use leocard_protocol::GuandanCommand;

fn reconnect(connection: u64) -> ClientCommand {
    let key = SigningKey::from_bytes(&[connection as u8; 32]);
    let mut request = JoinRequest {
        name: format!("玩家{connection}"),
        reconnect_token: ReconnectToken(connection),
        profile_id: ProfileId(key.verifying_key().to_bytes()),
        reference_points: 0,
        completed_games: 0,
        // Reconnect must retain host-owned statistics rather than replace them.
        game_profiles: PlayerGameProfiles::default(),
        identity_signature: Vec::new(),
        avatar_png: None,
    };
    request.identity_signature = key
        .sign(&request.identity_payload(ROOM))
        .to_bytes()
        .to_vec();
    ClientCommand::join(request)
}

#[test]
fn reconnect_restarts_the_human_window_and_preserves_host_owned_samples() {
    for unattended_millis in [0, 500] {
        let mut table = Table::new_for(GameRules::Guandan(Default::default()));
        // Avoid disconnecting the host, which intentionally closes the room.
        let mut actor = table.view().1.clocks[0].player;
        if actor == PlayerId(0) {
            let GameSnapshot::Guandan(s) = table.view_for(1).0 else {
                unreachable!()
            };
            table.send(
                1,
                ClientCommand::Game(
                    GuandanCommand::Act(GuandanAction::Play(vec![s.your_hand[0]])).into(),
                ),
            );
            actor = table.view().1.clocks[0].player;
        }
        let connection = u64::from(actor.0) + 1;
        table.session.advance_time(Duration::from_secs(3));
        table.session.disconnect(ConnectionId(connection));
        if unattended_millis > 0 {
            table
                .session
                .advance_time(Duration::from_millis(unattended_millis));
            assert!(table.view().1.clocks.is_empty());
        }
        let new_connection = connection + 10;
        table
            .requests
            .insert(new_connection, table.requests[&connection]);
        let rejoined = table.send(new_connection, reconnect(connection));
        assert!(
            !rejoined
                .iter()
                .any(|d| matches!(d.message.event, ServerEvent::Rejected { .. })),
            "{rejoined:?}"
        );
        let clock = table.view().1;
        assert_eq!(clock.clocks.len(), 1, "{rejoined:?}");
        assert_eq!(clock.clocks[0].elapsed_millis, 0);
        table.session.advance_time(Duration::from_secs(1));
        let GameSnapshot::Guandan(s) = table.view_for(new_connection).0 else {
            unreachable!()
        };
        let action = if s.table.is_some() {
            GuandanAction::Pass
        } else {
            GuandanAction::Play(vec![s.your_hand[0]])
        };
        table.send(
            new_connection,
            ClientCommand::Game(GuandanCommand::Act(action).into()),
        );
        let stats = table
            .view()
            .0
            .player(actor)
            .unwrap()
            .game_profiles
            .action_times
            .get(GameKind::Guandan);
        assert_eq!(
            (
                stats.total_millis,
                stats.completed_actions,
                stats.longest_millis
            ),
            (1000, 1, 1000)
        );
        table.session.disconnect(ConnectionId(new_connection));
        let last_connection = connection + 20;
        table
            .requests
            .insert(last_connection, table.requests[&new_connection]);
        let rejoined = table.send(last_connection, reconnect(connection));
        assert!(
            !rejoined
                .iter()
                .any(|d| matches!(d.message.event, ServerEvent::Rejected { .. })),
            "{rejoined:?}"
        );
        assert_eq!(
            table
                .view()
                .0
                .player(actor)
                .unwrap()
                .game_profiles
                .action_times
                .get(GameKind::Guandan),
            stats
        );
    }
}

#[test]
fn taking_over_from_auto_play_restarts_without_counting_unattended_time() {
    let mut table = Table::new_for(GameRules::Guandan(Default::default()));
    let actor = table.view().1.clocks[0].player;
    let connection = u64::from(actor.0) + 1;
    table.session.advance_time(Duration::from_secs(2));
    table.send(
        connection,
        ClientCommand::Game(GuandanCommand::SetAutoPlay { enabled: true }.into()),
    );
    assert!(table.view().1.clocks.is_empty());
    // Keep below the bot action delay so the same required decision remains.
    table.session.advance_time(Duration::from_millis(500));
    table.send(
        connection,
        ClientCommand::Game(GuandanCommand::SetAutoPlay { enabled: false }.into()),
    );
    assert_eq!(table.view().1.clocks[0].elapsed_millis, 0);
    table.session.advance_time(Duration::from_millis(750));
    let GameSnapshot::Guandan(snapshot) = table.view_for(connection).0 else {
        unreachable!()
    };
    table.send(
        connection,
        ClientCommand::Game(
            GuandanCommand::Act(leocard_guandan::GuandanAction::Play(vec![
                snapshot.your_hand[0],
            ]))
            .into(),
        ),
    );
    let stats = table
        .view()
        .0
        .player(actor)
        .unwrap()
        .game_profiles
        .action_times
        .get(GameKind::Guandan);
    assert_eq!(
        (
            stats.total_millis,
            stats.completed_actions,
            stats.longest_millis
        ),
        (750, 1, 750)
    );
}
