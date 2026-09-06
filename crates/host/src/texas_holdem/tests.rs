use std::collections::HashMap;

use ed25519_dalek::{Signer, SigningKey};
use leocard_protocol::{
    ClientCommand, GameCommand, GameSnapshot, JoinRequest, PROTOCOL_VERSION, PlayerGameProfiles,
    ProfileId, ReconnectToken, RequestId, Revision, SeatId, ServerEvent, ServerMessage,
    TexasHoldemCommand, TexasHoldemViolation, decode_frame, encode_frame, join_identity_payload,
};
use leocard_texas_holdem::{
    Phase, TexasHoldemAction, TexasHoldemBlindKind, TexasHoldemRank, TexasHoldemRuleSet,
    TexasHoldemStreet, TexasHoldemSuit, build_deck,
};

use super::*;

const ROOM: RoomId = RoomId(9527);
const HOST_CONNECTION: ConnectionId = ConnectionId(10);
const SECOND_CONNECTION: ConnectionId = ConnectionId(20);
const THIRD_CONNECTION: ConnectionId = ConnectionId(30);

fn message(request: u64, command: ClientCommand) -> ClientMessage {
    ClientMessage::new(ROOM, RequestId(request), command)
}

fn join_command(name: &str, token: u64) -> ClientCommand {
    let mut secret = [0; 32];
    secret[..8].copy_from_slice(&token.to_be_bytes());
    secret[8] = 9;
    let key = SigningKey::from_bytes(&secret);
    let reconnect_token = ReconnectToken(token);
    let profile_id = ProfileId(key.verifying_key().to_bytes());
    let game_profiles = PlayerGameProfiles::default();
    let payload = join_identity_payload(ROOM, reconnect_token, name, 0, 0, &game_profiles);
    ClientCommand::join(JoinRequest {
        name: name.to_owned(),
        reconnect_token,
        profile_id,
        reference_points: 0,
        completed_games: 0,
        game_profiles,
        identity_signature: key.sign(&payload).to_bytes().to_vec(),
    })
}

fn session_waiting_for_blinds() -> TexasHoldemSession {
    let mut session = TexasHoldemSession::new_with_host_port(
        ROOM,
        52301,
        TexasHoldemRuleSet::default(),
        build_deck(false),
    )
    .unwrap();
    for (connection, name, token) in [
        (HOST_CONNECTION, "房主", 1),
        (SECOND_CONNECTION, "玩家二", 2),
        (THIRD_CONNECTION, "玩家三", 3),
    ] {
        session.handle(connection, message(1, join_command(name, token)));
    }
    for connection in [SECOND_CONNECTION, THIRD_CONNECTION] {
        session.handle(
            connection,
            message(2, ClientCommand::SetReady { ready: true }),
        );
    }
    let deliveries = session.handle(HOST_CONNECTION, message(2, ClientCommand::StartGame));
    assert!(deliveries.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(_))
    )));
    session
}

fn started_session() -> TexasHoldemSession {
    let mut session = session_waiting_for_blinds();
    for _ in 0..2 {
        let player = session.game().unwrap().current_player().unwrap();
        session
            .game
            .as_mut()
            .unwrap()
            .act(player, TexasHoldemAction::PostBlind)
            .unwrap();
    }
    session
}

#[test]
fn lobby_start_waits_for_both_blind_actions() {
    let mut session = session_waiting_for_blinds();
    let first = session
        .game()
        .unwrap()
        .snapshot(session.room.players[0].id)
        .unwrap()
        .blind_to_post
        .unwrap();
    let connection = connection_for(&session, first.player);
    let delivered = session.handle(
        connection,
        message(
            3,
            ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                action: TexasHoldemAction::PostBlind,
            })),
        ),
    );
    assert!(delivered.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::GameEvent(GameEvent::TexasHoldem(TexasHoldemEvent::ActionApplied {
            action: TexasHoldemAction::PostBlind,
            ..
        }))
    )));
    assert!(session.game().unwrap().game().blind_to_post().is_some());
}

#[test]
fn texas_auto_play_waits_one_second_then_uses_the_passive_bot_action() {
    let mut session = session_waiting_for_blinds();
    let current = session.game().unwrap().current_player().unwrap();
    let connection = connection_for(&session, current);
    session.handle(
        connection,
        message(
            3,
            ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::SetAutoPlay {
                enabled: true,
            })),
        ),
    );

    let snapshot = session.game_snapshot(current);
    assert!(
        snapshot
            .players
            .iter()
            .find(|player| player.id == current)
            .unwrap()
            .auto_play
    );
    assert!(session.advance_time(Duration::from_millis(999)).is_empty());
    let deliveries = session.advance_time(Duration::from_millis(1));
    assert!(deliveries.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::GameEvent(GameEvent::TexasHoldem(
            TexasHoldemEvent::ActionApplied {
                player,
                action: TexasHoldemAction::PostBlind,
                ..
            }
        )) if player == current
    )));
}

#[cfg(feature = "developer")]
#[test]
fn developer_host_right_click_commands_add_default_texas_bots() {
    let mut session = TexasHoldemSession::new_with_host_port(
        ROOM,
        52301,
        TexasHoldemRuleSet::default(),
        build_deck(false),
    )
    .unwrap();
    session.handle(HOST_CONNECTION, message(1, join_command("房主", 1)));
    let rejected = session.handle(HOST_CONNECTION, message(2, ClientCommand::StartGame));
    assert!(rejected.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::Rejected {
            reason: RejectReason::NotEnoughPlayers { .. }
        }
    )));
    let empty_seats = (0..TABLE_SEAT_COUNT)
        .map(SeatId)
        .filter(|seat| {
            session
                .room
                .players
                .iter()
                .all(|player| player.seat != Some(*seat))
        })
        .take(2)
        .collect::<Vec<_>>();
    for (index, seat) in empty_seats.into_iter().enumerate() {
        session.handle(
            HOST_CONNECTION,
            message(
                3 + index as u64,
                ClientCommand::ConfigureBotSeat {
                    seat,
                    occupied: true,
                },
            ),
        );
    }
    let deliveries = session.handle(HOST_CONNECTION, message(5, ClientCommand::StartGame));

    assert!(deliveries.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(_))
    )));
    assert_eq!(
        session.room.players.len(),
        TexasHoldemRuleSet::MIN_PLAYERS as usize
    );
    assert_eq!(
        session
            .room
            .players
            .iter()
            .filter(|player| player.is_bot)
            .count(),
        TexasHoldemRuleSet::MIN_PLAYERS as usize - 1
    );
    for bot in session.room.players.iter().filter(|player| player.is_bot) {
        assert_eq!(bot.profile_id, ProfileId([0; 32]));
        assert_eq!(bot.avatar, None);
        assert_eq!(bot.reference_points, 0);
        assert_eq!(bot.completed_games, 0);
        assert!(bot.auto_play);
    }
    let snapshot = session.game_snapshot(session.room.host_player_id().unwrap());
    assert!(
        snapshot
            .players
            .iter()
            .filter(|player| player.auto_play)
            .count()
            >= 2
    );
}

fn connection_for(session: &TexasHoldemSession, player: PlayerId) -> ConnectionId {
    session
        .room
        .players
        .iter()
        .find(|participant| participant.id == player)
        .unwrap()
        .connection
}

#[test]
fn common_lobby_starts_texas_and_each_snapshot_has_only_its_own_hole_cards() {
    let session = started_session();
    assert_eq!(session.rules().player_count, TABLE_SEAT_COUNT);
    let game = session.game().unwrap();
    let snapshots = session
        .room
        .players
        .iter()
        .map(|player| game.snapshot(player.id).unwrap())
        .collect::<Vec<_>>();
    assert!(snapshots.iter().all(|snapshot| {
        snapshot.your_hole_cards.len() == 2 && snapshot.revealed_hands.is_empty()
    }));
    assert_ne!(snapshots[0].your_hole_cards, snapshots[1].your_hole_cards);
    assert!(snapshots.iter().all(|snapshot| snapshot.host_port == 52301));
    assert_eq!(snapshots[0].players.len(), 3);
    assert_eq!(snapshots[0].pot, 3);
}

#[test]
fn protocol_actions_use_the_adapter_and_rejections_leave_state_unchanged() {
    let mut session = started_session();
    let current = session.game().unwrap().current_player().unwrap();
    let connection = connection_for(&session, current);
    let before = session.game().unwrap().game().clone();
    let rejected = session.handle(
        connection,
        message(
            3,
            ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                action: TexasHoldemAction::Check,
            })),
        ),
    );
    assert!(rejected.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::Rejected {
            reason: RejectReason::GameViolation(GameViolation::TexasHoldem(
                TexasHoldemViolation::CannotCheckWhileFacingBet { .. }
            ))
        }
    )));
    assert_eq!(session.game().unwrap().game(), &before);

    let accepted = session.handle(
        connection,
        message(
            4,
            ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                action: TexasHoldemAction::Call,
            })),
        ),
    );
    assert!(accepted.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::GameEvent(GameEvent::TexasHoldem(
            TexasHoldemEvent::ActionApplied {
                player,
                action: TexasHoldemAction::Call,
                ..
            }
        )) if player == current
    )));
}

#[test]
fn all_players_ready_start_the_next_hand_with_stacks_and_rotated_dealer_preserved() {
    let mut session = started_session();
    let first_dealer = session.game().unwrap().game().dealer();
    let mut requests = HashMap::from([
        (HOST_CONNECTION, 2_u64),
        (SECOND_CONNECTION, 2_u64),
        (THIRD_CONNECTION, 2_u64),
    ]);
    while matches!(session.game().unwrap().game().phase(), Phase::Betting(_)) {
        let player = session.game().unwrap().current_player().unwrap();
        let connection = connection_for(&session, player);
        let request = requests.get_mut(&connection).unwrap();
        *request += 1;
        session.handle(
            connection,
            message(
                *request,
                ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                    action: TexasHoldemAction::Fold,
                })),
            ),
        );
    }
    assert!(matches!(
        session.game().unwrap().game().phase(),
        Phase::Complete(_)
    ));
    for (index, connection) in [HOST_CONNECTION, SECOND_CONNECTION, THIRD_CONNECTION]
        .into_iter()
        .enumerate()
    {
        let request = requests.get_mut(&connection).unwrap();
        *request += 1;
        session.handle(connection, message(*request, ClientCommand::PlayAgain));
        if index < 2 {
            let snapshot = session.game_snapshot(session.room.players[0].id);
            assert!(matches!(
                snapshot.phase,
                TexasHoldemPhaseView::HandComplete { .. }
            ));
            assert_eq!(
                snapshot
                    .players
                    .iter()
                    .filter(|player| player.ready)
                    .count(),
                index + 1
            );
        }
    }
    let game = session.game().unwrap().game();
    assert_eq!(game.hand_number(), 1);
    assert_ne!(game.dealer(), first_dealer);
    assert!(matches!(game.phase(), Phase::Betting(_)));
}

#[test]
fn first_busted_player_finishes_the_tournament_and_applies_shared_rating_once() {
    let prefix = [
        TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::King),
        TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Queen),
        TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Ace),
        TexasHoldemCard::new(TexasHoldemSuit::Heart, TexasHoldemRank::King),
        TexasHoldemCard::new(TexasHoldemSuit::Heart, TexasHoldemRank::Queen),
        TexasHoldemCard::new(TexasHoldemSuit::Heart, TexasHoldemRank::Ace),
        TexasHoldemCard::new(TexasHoldemSuit::Club, TexasHoldemRank::Two),
        TexasHoldemCard::new(TexasHoldemSuit::Diamond, TexasHoldemRank::Three),
        TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Seven),
        TexasHoldemCard::new(TexasHoldemSuit::Club, TexasHoldemRank::Eight),
        TexasHoldemCard::new(TexasHoldemSuit::Diamond, TexasHoldemRank::Nine),
    ];
    let deck = prefix
        .into_iter()
        .chain(
            build_deck(false)
                .into_iter()
                .filter(|card| !prefix.contains(card)),
        )
        .collect();
    let mut session = TexasHoldemSession::new_with_host_port(
        ROOM,
        52301,
        TexasHoldemRuleSet {
            starting_chips: 5,
            ..TexasHoldemRuleSet::default()
        },
        deck,
    )
    .unwrap();
    for (connection, name, token) in [
        (HOST_CONNECTION, "房主", 1),
        (SECOND_CONNECTION, "玩家二", 2),
        (THIRD_CONNECTION, "玩家三", 3),
    ] {
        session.handle(connection, message(1, join_command(name, token)));
    }
    for connection in [SECOND_CONNECTION, THIRD_CONNECTION] {
        session.handle(
            connection,
            message(2, ClientCommand::SetReady { ready: true }),
        );
    }
    session.handle(HOST_CONNECTION, message(2, ClientCommand::StartGame));
    let mut request = 3;
    while session.game().unwrap().game().blind_to_post().is_some() {
        let player = session.game().unwrap().current_player().unwrap();
        let connection = connection_for(&session, player);
        session.handle(
            connection,
            message(
                request,
                ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                    action: TexasHoldemAction::PostBlind,
                })),
            ),
        );
        request += 1;
    }
    while matches!(session.game().unwrap().game().phase(), Phase::Betting(_)) {
        let player = session.game().unwrap().current_player().unwrap();
        let connection = connection_for(&session, player);
        session.handle(
            connection,
            message(
                request,
                ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                    action: TexasHoldemAction::AllIn,
                })),
            ),
        );
        request += 1;
    }
    let snapshot = session.game_snapshot(session.room.players[0].id);
    let TexasHoldemPhaseView::HandComplete {
        tournament_complete,
        reference_changes,
        ..
    } = snapshot.phase
    else {
        panic!("all-in showdown should complete the hand");
    };
    assert!(
        tournament_complete,
        "final stacks: {:?}",
        snapshot
            .players
            .iter()
            .map(|player| player.stack)
            .collect::<Vec<_>>()
    );
    assert_eq!(reference_changes.len(), 3);
    assert!(snapshot.players.iter().any(|player| player.stack == 0));
    for player in &snapshot.players {
        let stats = player.game_profiles.texas_holdem.as_ref().unwrap();
        let delta = reference_changes
            .iter()
            .find(|change| change.player == player.id)
            .unwrap()
            .delta;
        let placement = 1 + snapshot
            .players
            .iter()
            .filter(|other| other.stack > player.stack)
            .count();
        assert_eq!(stats.completed_games, 1);
        assert_eq!(stats.total_reference_delta, i64::from(delta));
        assert_eq!(stats.total_final_chips, u64::from(player.stack));
        assert_eq!(stats.placement_counts[placement - 1], 1);
        assert_eq!(stats.hands_played, 1);
        assert_eq!(stats.all_in_actions, 1);
        assert_eq!(stats.wager_actions, 1);
        assert_eq!(stats.hand_category_counts.iter().sum::<u32>(), 1);
    }
    let points_after = session
        .room
        .players
        .iter()
        .map(|player| player.reference_points)
        .collect::<Vec<_>>();
    session.apply_finished_reference_points();
    assert_eq!(
        session
            .room
            .players
            .iter()
            .map(|player| player.reference_points)
            .collect::<Vec<_>>(),
        points_after
    );
}

#[test]
fn texas_profile_action_statistics_ignore_blinds_and_zero_value_actions() {
    let mut session =
        TexasHoldemSession::new(ROOM, TexasHoldemRuleSet::default(), build_deck(false)).unwrap();
    session.match_profile_stats = vec![TexasHoldemProfileStats::default(); 2];
    session.record_profile_events(&[
        TexasHoldemEvent::ActionApplied {
            player: PlayerId(0),
            action: TexasHoldemAction::PostBlind,
            amount: 2,
        },
        TexasHoldemEvent::ActionApplied {
            player: PlayerId(0),
            action: TexasHoldemAction::Check,
            amount: 0,
        },
        TexasHoldemEvent::ActionApplied {
            player: PlayerId(0),
            action: TexasHoldemAction::RaiseTo(12),
            amount: 10,
        },
        TexasHoldemEvent::ActionApplied {
            player: PlayerId(0),
            action: TexasHoldemAction::Fold,
            amount: 0,
        },
    ]);

    let stats = &session.match_profile_stats[0];
    assert_eq!(stats.voluntary_actions, 3);
    assert_eq!(stats.check_actions, 1);
    assert_eq!(stats.raise_actions, 1);
    assert_eq!(stats.hands_folded, 1);
    assert_eq!(stats.wager_actions, 1);
    assert_eq!(stats.wagered_chips, 10);
}

#[test]
fn disconnecting_the_current_guest_immediately_folds_them() {
    let mut session = started_session();
    let current = session.game().unwrap().current_player().unwrap();
    let connection = connection_for(&session, current);
    if connection == HOST_CONNECTION {
        // 先合法行动一次，确保测试目标是可断线而不关闭房间的客人。
        session.handle(
            connection,
            message(
                3,
                ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                    action: TexasHoldemAction::Call,
                })),
            ),
        );
    }
    let guest = session.game().unwrap().current_player().unwrap();
    let guest_connection = connection_for(&session, guest);
    assert_ne!(guest_connection, HOST_CONNECTION);
    let deliveries = session.disconnect(guest_connection);
    assert!(!deliveries.is_empty());
    let core_index = session
        .game()
        .unwrap()
        .players()
        .iter()
        .position(|player| player.id == guest)
        .unwrap();
    assert!(session.game().unwrap().game().players()[core_index].folded());
}

#[test]
fn texas_rules_are_visible_in_the_shared_lobby_snapshot() {
    let mut session = TexasHoldemSession::new(
        ROOM,
        TexasHoldemRuleSet {
            short_deck: true,
            ignore_kickers: true,
            omaha: true,
            ..TexasHoldemRuleSet::default()
        },
        build_deck(true),
    )
    .unwrap();
    let deliveries = session.handle(HOST_CONNECTION, message(1, join_command("房主", 11)));
    let lobby = deliveries
        .iter()
        .find_map(|delivery| match &delivery.message.event {
            ServerEvent::LobbySnapshot(snapshot) => Some(snapshot),
            _ => None,
        })
        .unwrap();
    assert_eq!(lobby.game, GameKind::TexasHoldem);
    assert!(lobby.texas_holdem_rules().unwrap().short_deck);
    assert!(lobby.texas_holdem_rules().unwrap().ignore_kickers);
    assert!(lobby.texas_holdem_rules().unwrap().omaha);
    assert_eq!(lobby.host_port, 52300);
    assert!(session.game().is_none());
}

#[test]
fn lobby_disconnect_releases_the_player_from_count_and_capacity() {
    let mut session =
        TexasHoldemSession::new(ROOM, TexasHoldemRuleSet::default(), build_deck(false)).unwrap();
    session.handle(HOST_CONNECTION, message(1, join_command("房主", 1)));
    session.handle(SECOND_CONNECTION, message(1, join_command("玩家二", 2)));
    session.handle(THIRD_CONNECTION, message(1, join_command("玩家三", 3)));

    let deliveries = session.disconnect(SECOND_CONNECTION);

    assert!(session.room.players[1].left);
    assert_eq!(
        session
            .room
            .players
            .iter()
            .filter(|player| !player.left)
            .count(),
        2
    );
    assert!(deliveries.iter().all(|delivery| {
        let ServerEvent::LobbySnapshot(snapshot) = &delivery.message.event else {
            return false;
        };
        snapshot.players.len() == 2 && snapshot.players.iter().all(|player| player.connected)
    }));

    let replacement = ConnectionId(40);
    let rejoined = session.handle(replacement, message(1, join_command("新玩家", 4)));
    assert!(rejoined.iter().any(|delivery| matches!(
        &delivery.message.event,
        ServerEvent::LobbySnapshot(snapshot) if snapshot.players.len() == 3
    )));
}
const HOST: PlayerId = PlayerId(20);
const LEFT: PlayerId = PlayerId(30);
const RIGHT: PlayerId = PlayerId(10);

fn player(id: PlayerId, seat: u8) -> TablePlayer {
    TablePlayer {
        id,
        profile_id: ProfileId([id.0; 32]),
        name: format!("玩家{}", id.0),
        avatar: None,
        seat: SeatId(seat),
        connected: true,
        reference_points: 0,
        completed_games: 0,
    }
}

fn raw_adapter() -> TexasHoldemAdapter {
    TexasHoldemAdapter::new(
        MatchId([7; 16]),
        52300,
        HOST,
        vec![player(RIGHT, 2), player(HOST, 0), player(LEFT, 1)],
        TexasHoldemRuleSet::default(),
        HOST,
        build_deck(false),
    )
    .unwrap()
}

fn adapter() -> TexasHoldemAdapter {
    let mut game = raw_adapter();
    game.act(LEFT, TexasHoldemAction::PostBlind).unwrap();
    game.act(RIGHT, TexasHoldemAction::PostBlind).unwrap();
    game
}

#[test]
fn blind_posting_is_exposed_before_normal_preflop_actions() {
    let mut game = raw_adapter();
    let first = game.snapshot(HOST).unwrap().blind_to_post.unwrap();
    assert_eq!(first.player, LEFT);
    assert_eq!(first.kind, TexasHoldemBlindKind::Small);
    assert_eq!(first.amount, 1);
    assert!(matches!(
        game.act(LEFT, TexasHoldemAction::Call),
        Err(AdapterError::Violation(TexasHoldemViolation::MustPostBlind))
    ));
    game.act(LEFT, TexasHoldemAction::PostBlind).unwrap();
    let second = game.snapshot(HOST).unwrap().blind_to_post.unwrap();
    assert_eq!(second.player, RIGHT);
    assert_eq!(second.kind, TexasHoldemBlindKind::Big);
    game.act(RIGHT, TexasHoldemAction::PostBlind).unwrap();
    assert!(game.snapshot(HOST).unwrap().blind_to_post.is_none());
    assert_eq!(game.current_player(), Some(HOST));
}

#[test]
fn seat_order_maps_platform_ids_to_core_positions() {
    let game = adapter();
    let snapshot = game.snapshot(HOST).unwrap();
    assert_eq!(
        snapshot.players.iter().map(|p| p.id).collect::<Vec<_>>(),
        vec![HOST, LEFT, RIGHT]
    );
    assert_eq!(snapshot.dealer, HOST);
    assert_eq!(snapshot.small_blind, LEFT);
    assert_eq!(snapshot.big_blind, RIGHT);
    assert_eq!(snapshot.current_player, Some(HOST));
}

#[test]
fn betting_snapshots_only_contain_the_recipient_hole_cards() {
    let game = adapter();
    let host = game.snapshot(HOST).unwrap();
    let left = game.snapshot(LEFT).unwrap();
    assert_eq!(host.your_hole_cards.len(), 2);
    assert_eq!(left.your_hole_cards.len(), 2);
    assert_ne!(host.your_hole_cards, left.your_hole_cards);
    assert!(host.revealed_hands.is_empty());
    assert!(left.revealed_hands.is_empty());
    assert_eq!(host.players, left.players);
}

#[test]
fn omaha_snapshots_deal_and_reveal_four_cards_with_an_evaluated_hand() {
    let mut game = TexasHoldemAdapter::new(
        MatchId([8; 16]),
        52300,
        HOST,
        vec![player(RIGHT, 2), player(HOST, 0), player(LEFT, 1)],
        TexasHoldemRuleSet {
            omaha: true,
            ..TexasHoldemRuleSet::default()
        },
        HOST,
        build_deck(false),
    )
    .unwrap();
    assert_eq!(game.snapshot(HOST).unwrap().your_hole_cards.len(), 4);

    game.act(LEFT, TexasHoldemAction::PostBlind).unwrap();
    game.act(RIGHT, TexasHoldemAction::PostBlind).unwrap();
    game.act(HOST, TexasHoldemAction::AllIn).unwrap();
    game.act(LEFT, TexasHoldemAction::AllIn).unwrap();
    game.act(RIGHT, TexasHoldemAction::Call).unwrap();

    let snapshot = game.snapshot(HOST).unwrap();
    assert_eq!(snapshot.community.len(), 5);
    assert_eq!(snapshot.revealed_hands.len(), 3);
    assert!(
        snapshot
            .revealed_hands
            .iter()
            .all(|hand| hand.cards.len() == 4 && hand.best.is_some())
    );

    let message = ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(523),
        revision: Revision(9),
        in_reply_to: None,
        event: ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(snapshot)),
    };
    let decoded: ServerMessage = decode_frame(&encode_frame(&message).unwrap()).unwrap();
    assert_eq!(decoded, message);
}

#[test]
fn adapter_emits_action_and_street_events() {
    let mut game = adapter();
    game.act(HOST, TexasHoldemAction::Call).unwrap();
    game.act(LEFT, TexasHoldemAction::Call).unwrap();
    let events = game.act(RIGHT, TexasHoldemAction::Check).unwrap();
    assert_eq!(events.len(), 2);
    assert!(matches!(
        events[0],
        TexasHoldemEvent::ActionApplied {
            player: RIGHT,
            action: TexasHoldemAction::Check,
            amount: 0,
        }
    ));
    assert!(matches!(
        &events[1],
        TexasHoldemEvent::StreetAdvanced { street, dealt }
            if *street == TexasHoldemStreet::Flop && dealt.len() == 3
    ));
    assert_eq!(game.snapshot(HOST).unwrap().community.len(), 3);
}

#[test]
fn rejected_action_returns_a_wire_violation_without_mutating_state() {
    let mut game = adapter();
    let before = game.game().clone();
    assert_eq!(
        game.act(LEFT, TexasHoldemAction::Call),
        Err(AdapterError::Violation(
            TexasHoldemViolation::NotPlayersTurn
        ))
    );
    assert_eq!(game.game(), &before);
}

#[test]
fn showdown_reveals_all_hands_and_exposes_pot_awards() {
    let mut game = adapter();
    game.act(HOST, TexasHoldemAction::AllIn).unwrap();
    game.act(LEFT, TexasHoldemAction::AllIn).unwrap();
    let events = game.act(RIGHT, TexasHoldemAction::Call).unwrap();
    assert!(matches!(
        events.last(),
        Some(TexasHoldemEvent::HandFinished { showdown: true })
    ));
    let snapshot = game.snapshot(HOST).unwrap();
    assert_eq!(snapshot.community.len(), 5);
    assert_eq!(snapshot.revealed_hands.len(), 3);
    let TexasHoldemPhaseView::HandComplete {
        showdown, awards, ..
    } = snapshot.phase
    else {
        panic!("the hand should be complete");
    };
    assert!(showdown);
    assert_eq!(awards.iter().map(|award| award.amount).sum::<u32>(), 60);
}

#[test]
fn showdown_keeps_previously_folded_hands_private() {
    let mut game = adapter();
    game.act(HOST, TexasHoldemAction::Fold).unwrap();
    game.act(LEFT, TexasHoldemAction::AllIn).unwrap();
    game.act(RIGHT, TexasHoldemAction::Call).unwrap();
    let snapshot = game.snapshot(HOST).unwrap();
    assert_eq!(snapshot.revealed_hands.len(), 2);
    assert!(
        snapshot
            .revealed_hands
            .iter()
            .all(|hand| hand.player != HOST)
    );
}

#[test]
fn uncontested_win_does_not_reveal_any_hole_cards() {
    let mut game = adapter();
    game.act(HOST, TexasHoldemAction::Fold).unwrap();
    let events = game.act(LEFT, TexasHoldemAction::Fold).unwrap();
    assert!(matches!(
        events.last(),
        Some(TexasHoldemEvent::HandFinished { showdown: false })
    ));
    let snapshot = game.snapshot(RIGHT).unwrap();
    assert!(snapshot.revealed_hands.is_empty());
}

#[test]
fn next_hand_keeps_stacks_and_rotates_the_dealer() {
    let mut game = adapter();
    game.act(HOST, TexasHoldemAction::Fold).unwrap();
    game.act(LEFT, TexasHoldemAction::Fold).unwrap();
    game.start_next_hand(build_deck(false)).unwrap();
    let snapshot = game.snapshot(HOST).unwrap();
    assert_eq!(snapshot.hand_number, 1);
    assert_eq!(snapshot.dealer, LEFT);
    assert!(matches!(
        snapshot.phase,
        TexasHoldemPhaseView::Betting {
            street: TexasHoldemStreet::PreFlop
        }
    ));
}

#[test]
fn private_snapshot_round_trips_through_the_wire_frame() {
    let snapshot = adapter().snapshot(HOST).unwrap();
    let message = ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(523),
        revision: Revision(8),
        in_reply_to: None,
        event: ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(snapshot.clone())),
    };
    let decoded: ServerMessage = decode_frame(&encode_frame(&message).unwrap()).unwrap();
    assert_eq!(decoded, message);
    let ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(decoded_snapshot)) = decoded.event
    else {
        panic!("wire frame changed the concrete game variant");
    };
    assert_eq!(decoded_snapshot.your_hole_cards, snapshot.your_hole_cards);
    assert!(decoded_snapshot.revealed_hands.is_empty());
}
