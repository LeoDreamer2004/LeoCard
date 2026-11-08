use ed25519_dalek::{Signer, SigningKey};
use leocard_blackjack::{BlackjackAction, BlackjackPhase, BlackjackRuleSet};
use leocard_doudizhu::{DoudizhuMode, DoudizhuRuleSet};
use leocard_game_common::PlayerId;
use leocard_host::{ConnectionId, Delivery, GameSetup, HostSession};
use leocard_protocol::{
    BlackjackCommand, ChatContent, ChatEmoji, ClientCommand, ClientMessage, GameKind, GameRules,
    GameSnapshot, JoinRequest, MahjongCommand, PlayerGameProfiles, PlayerInteractionKind,
    ProfileId, QiGui523Command, ReconnectToken, RejectReason, RequestId, RoomId,
    RoomModerationSnapshot, RoomViolation, SeatId, ServerEvent, UnoCommand,
};
use leocard_qigui523::QiGuiRuleSet;
use std::collections::HashMap;
use std::time::Duration;

const ROOM: RoomId = RoomId(91);
const OBSERVER: u64 = 99;

fn moderation_rules() -> [GameRules; 10] {
    [
        GameRules::QiGui523(QiGuiRuleSet {
            player_count: 4,
            ..Default::default()
        }),
        GameRules::Texas(Default::default()),
        GameRules::Shengji(Default::default()),
        GameRules::Uno(Default::default()),
        GameRules::Mahjong(Default::default()),
        GameRules::McGuess(Default::default()),
        GameRules::Blackjack(Default::default()),
        GameRules::Doudizhu(DoudizhuRuleSet {
            mode: DoudizhuMode::Four,
            ..Default::default()
        }),
        GameRules::Guandan(Default::default()),
        GameRules::MahjongLink(Default::default()),
    ]
}

fn moderation(deliveries: &[Delivery]) -> &RoomModerationSnapshot {
    deliveries
        .iter()
        .find_map(|delivery| match &delivery.message.event {
            ServerEvent::RoomModerationSnapshot(room) => Some(room),
            _ => None,
        })
        .expect("authoritative room controls and roster")
}

#[test]
fn host_controls_spectator_interactions_and_late_joiners_receive_the_policy_in_every_game() {
    for rules in moderation_rules() {
        let mut room = Room::started(rules);
        let forbidden = room.send(
            2,
            ClientCommand::SetSpectatorInteractionsBlocked { blocked: true },
        );
        assert!(matches!(
            forbidden[0].message.event,
            ServerEvent::Rejected {
                reason: RejectReason::Room(RoomViolation::OnlyHostCanConfigure)
            }
        ));
        let changed = room.send(
            1,
            ClientCommand::SetSpectatorInteractionsBlocked { blocked: true },
        );
        assert!(moderation(&changed).spectator_interactions_blocked);
        let joined = room.send(OBSERVER, join(99, OBSERVER));
        assert!(moderation(&joined).spectator_interactions_blocked);
        assert!(
            moderation(&joined)
                .participants
                .iter()
                .any(|player| player.spectating && player.name == "玩家99")
        );
        let interact = ClientCommand::Interact {
            target: PlayerId(0),
            kind: PlayerInteractionKind::Egg,
        };
        for kind in [
            PlayerInteractionKind::Egg,
            PlayerInteractionKind::EggBarrage,
        ] {
            let rejected = room.send(
                OBSERVER,
                ClientCommand::Interact {
                    target: PlayerId(0),
                    kind,
                },
            );
            assert!(matches!(
                rejected[0].message.event,
                ServerEvent::Rejected {
                    reason: RejectReason::Room(RoomViolation::SpectatorInteractionsBlocked)
                }
            ));
        }
        let chat = room.send(
            OBSERVER,
            ClientCommand::Chat {
                content: ChatContent::Text("好牌".to_owned()),
            },
        );
        assert!(
            chat.iter()
                .any(|delivery| matches!(delivery.message.event, ServerEvent::ChatMessage(_)))
        );
        let allowed = room.send(2, interact.clone());
        assert!(
            allowed.iter().any(|delivery| matches!(
                delivery.message.event,
                ServerEvent::PlayerInteraction(_)
            ))
        );
        room.send(
            1,
            ClientCommand::SetSpectatorInteractionsBlocked { blocked: false },
        );
        let allowed = room.send(OBSERVER, interact);
        assert!(
            allowed.iter().any(|delivery| matches!(
                delivery.message.event,
                ServerEvent::PlayerInteraction(_)
            ))
        );
        let disconnected = room.session.disconnect(ConnectionId(OBSERVER));
        assert!(
            moderation(&disconnected)
                .participants
                .iter()
                .any(|player| player.spectating && !player.connected)
        );
    }
}

#[test]
fn every_game_allows_the_host_to_kick_players_and_spectators_during_play() {
    for rules in moderation_rules() {
        let mut room = Room::started(rules);
        let joined = room.send(OBSERVER, join(99, OBSERVER));
        let observer = moderation(&joined)
            .participants
            .iter()
            .find(|player| player.spectating)
            .unwrap()
            .id;
        let forbidden = room.send(2, ClientCommand::KickPlayer { player: observer });
        assert!(matches!(
            forbidden[0].message.event,
            ServerEvent::Rejected {
                reason: RejectReason::Room(RoomViolation::OnlyHostCanKick)
            }
        ));
        for (target, connection) in [(observer, OBSERVER), (PlayerId(1), 2)] {
            let match_id = room.snapshot(1).match_id();
            let kicked = room.send(1, ClientCommand::KickPlayer { player: target });
            assert!(
                kicked
                    .iter()
                    .any(|delivery| delivery.recipient == ConnectionId(connection)
                        && matches!(delivery.message.event, ServerEvent::KickedFromRoom))
            );
            assert!(!room.session.is_current_connection(ConnectionId(connection)));
            assert!(
                !moderation(&kicked)
                    .participants
                    .iter()
                    .any(|player| player.id == target)
            );
            assert!(!kicked.iter().any(|delivery| matches!(
                delivery.message.event,
                ServerEvent::LobbySnapshot(_)
                    | ServerEvent::Rejected { .. }
                    | ServerEvent::PlayerLeft { .. }
            )));
            let rejected = room.send(connection, ClientCommand::RequestSnapshot);
            assert!(matches!(
                rejected[0].message.event,
                ServerEvent::Rejected { .. }
            ));
            assert!(!room.session.is_closed());
            assert_eq!(
                room.snapshot(1).match_id(),
                match_id,
                "kicking must not restart the match"
            );
        }
    }
}

#[test]
fn uno_and_mahjong_finish_with_kicked_seats_under_automatic_control() {
    for rules in [
        GameRules::Uno(Default::default()),
        GameRules::Mahjong(Default::default()),
    ] {
        let kind = rules.kind();
        let mut room = Room::started(rules);
        for player in 1..4 {
            room.send(
                1,
                ClientCommand::KickPlayer {
                    player: PlayerId(player),
                },
            );
        }
        let command = match kind {
            GameKind::Uno => UnoCommand::SetAutoPlay { enabled: true }.into(),
            _ => MahjongCommand::SetAutoPlay { enabled: true }.into(),
        };
        room.send(1, ClientCommand::Game(command));
        for _ in 0..2000 {
            room.session.advance_time(Duration::from_secs(5));
            if room.snapshot(1).match_complete() {
                break;
            }
        }
        assert!(
            room.snapshot(1).match_complete(),
            "{kind:?} stalled after a kick"
        );
    }
}

struct Room {
    session: HostSession,
    requests: HashMap<u64, u64>,
}

impl Room {
    fn started(rules: GameRules) -> Self {
        let mut room = Self {
            session: HostSession::new(ROOM, GameSetup::shuffled(52300, rules)).unwrap(),
            requests: HashMap::new(),
        };
        for connection in 1..=4 {
            room.send(connection, join(connection as u8, connection));
            room.send(
                connection,
                ClientCommand::SelectSeat {
                    seat: SeatId(connection as u8 - 1),
                },
            );
            room.send(connection, ClientCommand::SetReady { ready: true });
        }
        let deliveries = room.send(1, ClientCommand::StartGame);
        assert!(
            deliveries
                .iter()
                .any(|delivery| matches!(delivery.message.event, ServerEvent::GameStarted { .. })),
            "{deliveries:?}"
        );
        room
    }

    fn send(&mut self, connection: u64, command: ClientCommand) -> Vec<Delivery> {
        let request = self.requests.entry(connection).or_default();
        *request += 1;
        self.session.handle(
            ConnectionId(connection),
            ClientMessage::new(ROOM, RequestId(*request), command),
        )
    }

    fn finish_blackjack_hand(&mut self) {
        for _ in 0..100 {
            let snapshot = self.snapshot(1).into_blackjack().unwrap();
            if snapshot.phase == BlackjackPhase::Complete {
                break;
            }
            for connection in 1..=4 {
                let snapshot = self.snapshot(connection).into_blackjack().unwrap();
                if snapshot.current_player != Some(PlayerId(connection as u8 - 1)) {
                    continue;
                }
                let action = if snapshot.phase == BlackjackPhase::Betting {
                    BlackjackAction::Bet { amount: 2 }
                } else {
                    BlackjackAction::Stand
                };
                self.send(
                    connection,
                    ClientCommand::Game(BlackjackCommand::Act { action }.into()),
                );
            }
            self.session.advance_time(Duration::from_secs(5));
        }
    }

    fn snapshot(&mut self, connection: u64) -> GameSnapshot {
        game_snapshot(
            self.send(connection, ClientCommand::RequestSnapshot),
            connection,
        )
    }
}

fn join(identity: u8, token: u64) -> ClientCommand {
    let key = SigningKey::from_bytes(&[identity; 32]);
    let mut request = JoinRequest {
        name: format!("玩家{identity}"),
        reconnect_token: ReconnectToken(token),
        profile_id: ProfileId(key.verifying_key().to_bytes()),
        reference_points: 0,
        completed_games: 0,
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

fn game_snapshot(deliveries: Vec<Delivery>, connection: u64) -> GameSnapshot {
    deliveries
        .into_iter()
        .find_map(|delivery| match delivery.message.event {
            ServerEvent::GameSnapshot(snapshot)
                if delivery.recipient == ConnectionId(connection) =>
            {
                Some(snapshot)
            }
            _ => None,
        })
        .expect("recipient receives a game snapshot")
}

#[test]
fn midgame_spectators_share_selected_player_views_but_cannot_play_in_all_games() {
    for rules in [
        GameRules::QiGui523(QiGuiRuleSet {
            player_count: 4,
            ..Default::default()
        }),
        GameRules::Guandan(Default::default()),
        GameRules::Texas(Default::default()),
        GameRules::Shengji(Default::default()),
        GameRules::Uno(Default::default()),
        GameRules::Mahjong(Default::default()),
        GameRules::McGuess(Default::default()),
        GameRules::Blackjack(Default::default()),
        GameRules::Doudizhu(DoudizhuRuleSet {
            mode: DoudizhuMode::Four,
            ..Default::default()
        }),
    ] {
        let mut room = Room::started(rules);
        room.session.advance_time(Duration::from_secs(2));
        let joined = room.send(OBSERVER, join(99, OBSERVER));
        assert!(
            joined
                .iter()
                .any(|delivery| delivery.recipient == ConnectionId(1)
                    && matches!(&delivery.message.event, ServerEvent::SpectatorJoined { name }
                if name == "玩家99"))
        );
        let spectator = joined
            .iter()
            .find_map(|delivery| match delivery.message.event {
                ServerEvent::Joined { you } if delivery.recipient == ConnectionId(OBSERVER) => {
                    Some(you)
                }
                _ => None,
            })
            .unwrap();
        let snapshot = game_snapshot(joined, OBSERVER);
        assert_eq!(snapshot, room.snapshot(1));
        assert_ne!(spectator, snapshot.you());
        assert!(!snapshot.match_complete());
        for connection in 1..=4 {
            let target = room.snapshot(connection);
            let switched = room.send(
                OBSERVER,
                ClientCommand::SpectatePlayer {
                    player: target.you(),
                },
            );
            assert_eq!(game_snapshot(switched, OBSERVER), target);
        }
        let invalid = room.send(
            OBSERVER,
            ClientCommand::SpectatePlayer { player: spectator },
        );
        assert!(
            invalid
                .iter()
                .any(|delivery| matches!(delivery.message.event, ServerEvent::Rejected { .. }))
        );
        let forbidden = room.send(
            1,
            ClientCommand::SpectatePlayer {
                player: PlayerId(1),
            },
        );
        assert!(
            forbidden
                .iter()
                .any(|delivery| matches!(delivery.message.event, ServerEvent::Rejected { .. }))
        );
        let interactions = room.send(
            OBSERVER,
            ClientCommand::Interact {
                target: PlayerId(0),
                kind: PlayerInteractionKind::Flower,
            },
        );
        assert!(interactions.iter().any(|delivery| matches!(&delivery.message.event,
            ServerEvent::PlayerInteraction(event) if event.source == spectator && event.target == PlayerId(0))));
        assert_eq!(room.snapshot(OBSERVER), room.snapshot(4));
        for command in [
            ClientCommand::Game(QiGui523Command::Pass.into()),
            ClientCommand::PlayAgain,
            ClientCommand::SetReady { ready: true },
        ] {
            let rejected = room.send(OBSERVER, command);
            assert!(matches!(
                &rejected[0].message.event,
                ServerEvent::Rejected {
                    reason: RejectReason::Room(RoomViolation::SpectatorCannotAct)
                }
            ));
        }
        room.snapshot(1);
        assert!(room.session.is_current_connection(ConnectionId(OBSERVER)));
        assert!(
            room.send(OBSERVER, ClientCommand::LeaveRoom)
                .iter()
                .any(|delivery| matches!(delivery.message.event, ServerEvent::LeftRoom))
        );
        assert!(!room.session.is_current_connection(ConnectionId(OBSERVER)));
    }
}

#[test]
fn spectators_can_broadcast_attributed_text_but_not_emoji_or_quick_voice() {
    let mut room = Room::started(GameRules::QiGui523(QiGuiRuleSet {
        player_count: 4,
        ..Default::default()
    }));
    let spectator = room
        .send(OBSERVER, join(99, OBSERVER))
        .into_iter()
        .find_map(|delivery| match delivery.message.event {
            ServerEvent::Joined { you } if delivery.recipient == ConnectionId(OBSERVER) => {
                Some(you)
            }
            _ => None,
        })
        .unwrap();
    let deliveries = room.send(
        OBSERVER,
        ClientCommand::Chat {
            content: ChatContent::Text("  好牌  ".to_owned()),
        },
    );
    assert_eq!(deliveries.len(), 5);
    for connection in [1, 2, 3, 4, OBSERVER] {
        let delivery = deliveries
            .iter()
            .find(|delivery| delivery.recipient == ConnectionId(connection))
            .unwrap();
        let ServerEvent::ChatMessage(message) = &delivery.message.event else {
            panic!("expected a broadcast chat message");
        };
        assert_eq!(message.source, spectator);
        assert_eq!(message.spectator_name.as_deref(), Some("玩家99"));
        assert_eq!(message.content, ChatContent::Text("好牌".to_owned()));
    }
    for content in [
        ChatContent::Emoji(ChatEmoji::Laugh),
        ChatContent::QuickVoice(0),
    ] {
        let rejected = room.send(OBSERVER, ClientCommand::Chat { content });
        assert!(
            matches!(rejected.as_slice(), [delivery] if delivery.recipient == ConnectionId(OBSERVER)
            && matches!(&delivery.message.event, ServerEvent::Rejected {
                reason: RejectReason::Room(RoomViolation::SpectatorTextChatOnly),
            }))
        );
    }
}

#[test]
fn verified_reconnect_restores_the_original_player_instead_of_becoming_an_observer() {
    let mut room = Room::started(GameRules::QiGui523(QiGuiRuleSet {
        player_count: 4,
        ..Default::default()
    }));
    let before = room.snapshot(2);
    room.session.disconnect(ConnectionId(2));
    let resumed = game_snapshot(room.send(52, join(2, 502)), 52);
    assert_eq!(resumed.you(), before.you());
    assert_eq!(resumed.match_id(), before.match_id());
    assert!(!resumed.qigui523().unwrap().your_hand.is_empty());
    assert!(!room.session.is_current_connection(ConnectionId(2)));
    assert!(room.session.is_current_connection(ConnectionId(52)));
    let observer = game_snapshot(room.send(OBSERVER, join(99, OBSERVER)), OBSERVER);
    room.session.disconnect(ConnectionId(OBSERVER));
    let observer_resumed = game_snapshot(room.send(199, join(99, 199)), 199);
    assert_eq!(observer.you(), observer_resumed.you());
    assert_eq!(observer_resumed, room.snapshot(1));
    let rejected = room.send(53, join(2, 503));
    assert!(matches!(
        rejected[0].message.event,
        ServerEvent::Rejected { .. }
    ));
}

#[test]
fn observers_enter_the_lobby_and_get_seats_before_the_next_match() {
    let mut room = Room::started(GameRules::Blackjack(BlackjackRuleSet {
        rounds: 1,
        ..Default::default()
    }));
    room.send(OBSERVER, join(99, OBSERVER));
    room.finish_blackjack_hand();
    assert!(room.snapshot(1).match_complete());
    let deliveries = room.send(1, ClientCommand::PlayAgain);
    let observer_id = deliveries
        .iter()
        .find_map(|delivery| match delivery.message.event {
            ServerEvent::Joined { you } if delivery.recipient == ConnectionId(OBSERVER) => {
                Some(you)
            }
            _ => None,
        })
        .expect("observer receives their new seated identity");
    assert!(observer_id.0 < 128);
    let lobby = deliveries
        .iter()
        .find_map(|delivery| match &delivery.message.event {
            ServerEvent::LobbySnapshot(lobby) if delivery.recipient == ConnectionId(OBSERVER) => {
                Some(lobby)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(lobby.players.len(), 5);
    let observer = lobby
        .players
        .iter()
        .find(|player| player.player.id == observer_id)
        .unwrap();
    assert!(observer.seat.is_some() && !observer.ready);
    assert_eq!(observer.player.completed_games, 0);
    room.send(OBSERVER, ClientCommand::SetReady { ready: true });
    for connection in 1..=4 {
        room.send(connection, ClientCommand::SetReady { ready: true });
    }
    let restarted = room.send(1, ClientCommand::StartGame);
    assert_eq!(
        game_snapshot(restarted, OBSERVER)
            .blackjack()
            .unwrap()
            .players
            .len(),
        5
    );
}

#[test]
fn an_observer_does_not_interrupt_a_match_between_hands() {
    let mut room = Room::started(GameRules::Blackjack(BlackjackRuleSet {
        rounds: 2,
        ..Default::default()
    }));
    room.send(OBSERVER, join(99, OBSERVER));
    room.finish_blackjack_hand();
    assert!(!room.snapshot(1).match_complete());
    for connection in 1..=4 {
        let deliveries = room.send(connection, ClientCommand::PlayAgain);
        assert!(
            !deliveries
                .iter()
                .any(|delivery| matches!(delivery.message.event, ServerEvent::LobbySnapshot(_)))
        );
    }
    let next = room.snapshot(OBSERVER).into_blackjack().unwrap();
    assert_eq!(next.round, 2);
    assert_eq!(next.you, room.snapshot(1).you());
    assert_eq!(next.players.len(), 4);
}
