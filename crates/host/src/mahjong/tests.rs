use super::{MAHJONG_DEAL_INTERVAL, MahjongSession};
use crate::ConnectionId;
use ed25519_dalek::{Signer, SigningKey};
use leocard_mahjong::{
    Fan, FanValue, MahjongPlayerId, MahjongRuleSet, MahjongScoreResult, MahjongSuit, MahjongTile,
    MahjongTileKind, Phase, build_deck,
};
use leocard_protocol::{
    ClientCommand, ClientMessage, GameCommand, GameSnapshot, JoinRequest, MahjongCommand,
    MahjongEvent, MahjongHandResultView, MahjongPhaseView, MahjongSnapshot, MahjongWinView,
    PlayerGameProfiles, PlayerId, ProfileId, ReconnectToken, RequestId, RoomId, SeatId,
    ServerEvent, join_identity_payload,
};
use std::collections::HashSet;
use std::time::Duration;

const ROOM: RoomId = RoomId(2014);

#[test]
fn match_profile_counts_major_fan_draw_and_false_win_once() {
    let mut session =
        MahjongSession::new(ROOM, 52300, MahjongRuleSet::default(), build_deck()).unwrap();
    for index in 0..4 {
        session.handle(
            ConnectionId(index + 1),
            message(1, join_command(&format!("玩家{index}"), index + 1)),
        );
    }
    let tile = build_deck()[0];
    session.record_statistics(&[
        MahjongEvent::FalseWin {
            player: PlayerId(2),
            deltas: [10, 10, -30, 10],
        },
        MahjongEvent::HandFinished {
            result: MahjongHandResultView {
                winners: vec![MahjongWinView {
                    player: PlayerId(0),
                    from: Some(PlayerId(1)),
                    winning_tile: tile,
                    score: MahjongScoreResult {
                        fans: vec![FanValue {
                            fan: Fan::BigFourWinds,
                            count: 1,
                            points: 88,
                        }],
                        points_without_flowers: 88,
                        flower_points: 0,
                        total_points: 88,
                    },
                }],
                exhaustive_draw: false,
                deltas: [104, -88, -8, -8],
                match_scores: [104, -88, -8, -8],
                match_complete: true,
                sequence_index: 0,
                reference_changes: Vec::new(),
            },
        },
    ]);
    let winner = session.room.players[0]
        .game_profiles
        .mahjong
        .as_ref()
        .unwrap();
    assert_eq!(
        (
            winner.wins,
            winner.total_win_fan,
            winner.major_fan_counts[0]
        ),
        (1, 88, 1)
    );
    assert_eq!(
        session.room.players[1]
            .game_profiles
            .mahjong
            .as_ref()
            .unwrap()
            .discards_into_win,
        1
    );
    assert_eq!(
        session.room.players[2]
            .game_profiles
            .mahjong
            .as_ref()
            .unwrap()
            .false_wins,
        1
    );
    assert_eq!(
        session.finished_reference_changes.as_ref().unwrap().len(),
        4
    );
}

fn message(request: u64, command: ClientCommand) -> ClientMessage {
    ClientMessage::new(ROOM, RequestId(request), command)
}

fn join_command(name: &str, token: u64) -> ClientCommand {
    let mut secret = [0; 32];
    secret[..8].copy_from_slice(&token.to_be_bytes());
    secret[8] = 14;
    let key = SigningKey::from_bytes(&secret);
    let reconnect_token = ReconnectToken(token);
    let game_profiles = PlayerGameProfiles::default();
    let payload = join_identity_payload(ROOM, reconnect_token, name, 0, 0, &game_profiles);
    ClientCommand::join(JoinRequest {
        name: name.to_owned(),
        reconnect_token,
        profile_id: ProfileId(key.verifying_key().to_bytes()),
        reference_points: 0,
        completed_games: 0,
        game_profiles,
        identity_signature: key.sign(&payload).to_bytes().to_vec(),
    })
}

fn dealer_winning_deck() -> Vec<MahjongTile> {
    let kinds = [
        (1, 0),
        (1, 1),
        (1, 2),
        (2, 0),
        (2, 1),
        (2, 2),
        (3, 0),
        (3, 1),
        (3, 2),
        (4, 0),
        (4, 1),
        (4, 2),
        (5, 0),
        (5, 1),
    ];
    let selected = kinds
        .into_iter()
        .map(|(rank, copy)| {
            MahjongTile::new(MahjongTileKind::suited(MahjongSuit::Characters, rank), copy)
        })
        .collect::<Vec<_>>();
    let selected_set = selected.iter().copied().collect::<HashSet<_>>();
    let mut remaining = build_deck()
        .into_iter()
        .filter(|tile| !selected_set.contains(tile))
        .collect::<Vec<_>>();
    let mut deck = Vec::with_capacity(144);
    for round in 0..3 {
        deck.extend_from_slice(&selected[round * 4..round * 4 + 4]);
        for _ in 0..3 {
            deck.extend(remaining.drain(..4));
        }
    }
    deck.push(selected[12]);
    deck.extend(remaining.drain(..3));
    deck.push(selected[13]);
    deck.extend(remaining);
    deck
}

fn finish_server_deal(session: &mut MahjongSession) {
    while matches!(
        session.game().expect("game remains active").phase(),
        Phase::Dealing { .. }
    ) {
        assert!(!session.advance_time(MAHJONG_DEAL_INTERVAL).is_empty());
    }
    assert!(matches!(
        session.game().expect("game remains active").phase(),
        Phase::Playing
    ));
}

#[test]
fn four_clients_receive_private_views_and_continue_at_the_table() {
    let mut session = MahjongSession::new(
        ROOM,
        52300,
        MahjongRuleSet::default(),
        dealer_winning_deck(),
    )
    .expect("test wall is valid");
    let connections = [
        ConnectionId(10),
        ConnectionId(20),
        ConnectionId(30),
        ConnectionId(40),
    ];
    for (index, connection) in connections.into_iter().enumerate() {
        session.handle(
            connection,
            message(1, join_command(&format!("玩家{index}"), index as u64 + 1)),
        );
    }
    for (index, player) in session.room.players.iter_mut().enumerate() {
        player.seat = Some(SeatId(index as u8));
    }
    for connection in connections.into_iter().skip(1) {
        session.handle(
            connection,
            message(2, ClientCommand::SetReady { ready: true }),
        );
    }
    let deliveries = session.handle(connections[0], message(2, ClientCommand::StartGame));
    let snapshots = deliveries
        .iter()
        .filter_map(|delivery| match &delivery.message.event {
            ServerEvent::GameSnapshot(GameSnapshot::Mahjong(snapshot)) => {
                Some((delivery.recipient, snapshot))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(snapshots.len(), 4);
    for (_, snapshot) in &snapshots {
        assert_eq!(snapshot.players.len(), 4);
        assert!(matches!(
            snapshot.phase,
            MahjongPhaseView::Dealing { batch: 0 }
        ));
        assert_eq!(snapshot.wall_len, 144);
        assert!(snapshot.your_hand.is_empty());
        assert_eq!(
            snapshot.your_hand.len(),
            usize::from(snapshot.players[snapshot.you.0 as usize].concealed_count)
        );
        assert!(
            snapshot
                .players
                .iter()
                .all(|player| { player.id == snapshot.you || player.revealed_hand.is_none() })
        );
    }

    let dealer_connection = snapshots
        .iter()
        .find_map(|(connection, snapshot)| (snapshot.you == PlayerId(0)).then_some(*connection))
        .expect("one connection owns the dealer seat");
    let first_batch = session.advance_time(MAHJONG_DEAL_INTERVAL);
    let dealer_snapshot = first_batch
        .iter()
        .find_map(|delivery| match &delivery.message.event {
            ServerEvent::GameSnapshot(GameSnapshot::Mahjong(snapshot))
                if delivery.recipient == dealer_connection =>
            {
                Some(snapshot)
            }
            _ => None,
        })
        .expect("the first real batch is broadcast to the dealer");
    assert!(matches!(
        dealer_snapshot.phase,
        MahjongPhaseView::Dealing { batch: 1 }
    ));
    assert_eq!(dealer_snapshot.wall_len, 140);
    assert_eq!(dealer_snapshot.your_hand.len(), 4);
    assert_eq!(dealer_snapshot.players[0].concealed_count, 4);
    finish_server_deal(&mut session);
    let finished = session.handle(
        dealer_connection,
        message(
            10,
            ClientCommand::Game(GameCommand::Mahjong(MahjongCommand::DeclareSelfDraw)),
        ),
    );
    assert!(finished.iter().any(|delivery| matches!(
        &delivery.message.event,
        ServerEvent::GameSnapshot(GameSnapshot::Mahjong(MahjongSnapshot {
            phase: MahjongPhaseView::Finished { .. },
            ..
        }))
    )));
    let MahjongPhaseView::Finished { result } = &session.game_snapshot(PlayerId(0)).phase else {
        panic!("self draw finishes the single-hand match");
    };
    assert!(result.match_complete);
    assert_eq!(result.reference_changes.len(), 4);
    for player in &session.room.players {
        let stats = player.game_profiles.mahjong.as_ref().unwrap();
        assert_eq!(stats.completed_games, 1);
        assert_eq!(stats.hands_played, 1);
        assert_eq!(stats.wins, u32::from(player.id == PlayerId(0)));
        assert_eq!(stats.self_draws, u32::from(player.id == PlayerId(0)));
        assert_eq!(
            stats.total_match_score,
            i64::from(result.match_scores[usize::from(player.id.0)])
        );
        assert_eq!(
            stats.total_reference_delta,
            i64::from(result.reference_changes[usize::from(player.id.0)].delta)
        );
    }

    let mut lobby_session = session.clone();
    let returned = lobby_session.handle(connections[1], message(19, ClientCommand::ReturnToLobby));
    assert!(lobby_session.game().is_none());
    assert!(
        returned
            .iter()
            .any(|delivery| matches!(delivery.message.event, ServerEvent::LobbySnapshot(_)))
    );

    let original_match = session.match_id;
    let mut last = Vec::new();
    for connection in connections {
        last = session.handle(connection, message(20, ClientCommand::PlayAgain));
    }
    assert_ne!(session.match_id, original_match);
    assert!(last.iter().any(|delivery| matches!(
        &delivery.message.event,
        ServerEvent::GameSnapshot(GameSnapshot::Mahjong(MahjongSnapshot {
            phase: MahjongPhaseView::Dealing { batch: 0 },
            ..
        }))
    )));
    assert_eq!(
        session.game().expect("next hand started").sequence_index(),
        1
    );
}

#[test]
fn robot_waits_then_takes_an_automatic_action() {
    let mut session = MahjongSession::new(ROOM, 52300, MahjongRuleSet::default(), build_deck())
        .expect("standard wall is valid");
    let connections = [
        ConnectionId(110),
        ConnectionId(120),
        ConnectionId(130),
        ConnectionId(140),
    ];
    for (index, connection) in connections.into_iter().enumerate() {
        session.handle(
            connection,
            message(
                100 + index as u64,
                join_command(&format!("玩家{index}"), 20 + index as u64),
            ),
        );
    }
    for (index, player) in session.room.players.iter_mut().enumerate() {
        player.seat = Some(SeatId(index as u8));
        player.ready = true;
    }
    session.handle(connections[0], message(200, ClientCommand::StartGame));
    session.room.players[0].is_bot = true;
    session.room.players[0].auto_play = true;
    finish_server_deal(&mut session);

    assert!(session.advance_time(Duration::from_millis(999)).is_empty());
    let deliveries = session.advance_time(Duration::from_millis(1));

    assert!(!deliveries.is_empty());
    let game = session.game().expect("game remains active");
    let dealer = game.player(MahjongPlayerId(0)).unwrap();
    assert!(!game.discards().is_empty() || !dealer.melds().is_empty());
}

#[test]
fn player_can_toggle_mahjong_auto_play_and_cancel_a_pending_action() {
    let mut session = MahjongSession::new(ROOM, 52300, MahjongRuleSet::default(), build_deck())
        .expect("standard wall is valid");
    let connections = [
        ConnectionId(210),
        ConnectionId(220),
        ConnectionId(230),
        ConnectionId(240),
    ];
    for (index, connection) in connections.into_iter().enumerate() {
        session.handle(
            connection,
            message(
                index as u64 + 1,
                join_command(&format!("玩家{index}"), index as u64 + 30),
            ),
        );
    }
    for (index, player) in session.room.players.iter_mut().enumerate() {
        player.seat = Some(SeatId(index as u8));
        player.ready = true;
    }
    session.handle(connections[0], message(10, ClientCommand::StartGame));
    finish_server_deal(&mut session);

    let set_auto_play = |enabled| {
        ClientCommand::Game(GameCommand::Mahjong(MahjongCommand::SetAutoPlay {
            enabled,
        }))
    };
    let enabled = session.handle(connections[0], message(11, set_auto_play(true)));
    assert!(enabled.iter().any(|delivery| {
        matches!(
            &delivery.message.event,
            ServerEvent::GameSnapshot(GameSnapshot::Mahjong(snapshot))
                if delivery.recipient == connections[1]
                    && snapshot.players[0].auto_play
        )
    }));
    assert!(session.advance_time(Duration::from_millis(999)).is_empty());

    let disabled = session.handle(connections[0], message(12, set_auto_play(false)));
    assert!(disabled.iter().any(|delivery| {
        matches!(
            &delivery.message.event,
            ServerEvent::GameSnapshot(GameSnapshot::Mahjong(snapshot))
                if delivery.recipient == connections[1]
                    && !snapshot.players[0].auto_play
        )
    }));
    assert!(session.advance_time(Duration::from_secs(1)).is_empty());
    assert!(session.game().unwrap().discards().is_empty());

    session.handle(connections[0], message(13, set_auto_play(true)));
    let before_action = session.revision();
    assert!(!session.advance_time(Duration::from_secs(1)).is_empty());
    assert_ne!(session.revision(), before_action);
}
