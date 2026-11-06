#[path = "tests/timing.rs"]
mod timing;

use super::GuandanSession;
use crate::{ConnectionId, Delivery};
use ed25519_dalek::{Signer, SigningKey};
use leocard_game_common::PlayerId;
use leocard_guandan::{
    GuandanAction, GuandanCard, GuandanGameResult, GuandanGameState, GuandanMatchState,
    GuandanPhase, GuandanProfileStats, GuandanRuleSet, GuandanSuit, auto_play_action, build_deck,
    classify,
};
use leocard_protocol::{
    ClientCommand, ClientMessage, GameEvent, GameSnapshot, GameViolation, GuandanCommand,
    GuandanEvent, GuandanSnapshot, GuandanViolation, JoinRequest, PlayerGameProfiles, ProfileId,
    ReconnectToken, RejectReason, RequestId, RoomId, SeatId, ServerEvent,
};

fn send(session: &mut GuandanSession, connection: u64, command: ClientCommand) -> Vec<Delivery> {
    let connection = ConnectionId(connection);
    let request = session
        .room
        .last_requests
        .get(&connection)
        .map_or(1, |request| request.0 + 1);
    session.handle(
        connection,
        ClientMessage::new(RoomId(99), RequestId(request), command),
    )
}

fn join(connection: u64) -> ClientCommand {
    let key = SigningKey::from_bytes(&[connection as u8; 32]);
    let mut request = JoinRequest {
        name: format!("玩家{connection}"),
        reconnect_token: ReconnectToken(connection),
        profile_id: ProfileId(key.verifying_key().to_bytes()),
        reference_points: 0,
        completed_games: 0,
        game_profiles: PlayerGameProfiles::default(),
        identity_signature: Vec::new(),
        avatar_png: None,
    };
    request.identity_signature = key
        .sign(&request.identity_payload(RoomId(99)))
        .to_bytes()
        .to_vec();
    ClientCommand::join(request)
}

fn started() -> GuandanSession {
    let mut session =
        GuandanSession::new(RoomId(99), 52300, GuandanRuleSet::default(), build_deck()).unwrap();
    for connection in 1..=4 {
        send(&mut session, connection, join(connection));
        send(
            &mut session,
            connection,
            ClientCommand::SelectSeat {
                seat: SeatId(connection as u8 - 1),
            },
        );
        send(
            &mut session,
            connection,
            ClientCommand::SetReady { ready: true },
        );
    }
    let result = send(&mut session, 1, ClientCommand::StartGame);
    assert!(
        result
            .iter()
            .all(|delivery| !matches!(delivery.message.event, ServerEvent::Rejected { .. }))
    );
    session
}

#[test]
fn rejects_out_of_turn_action_without_changing_private_hands() {
    let mut session = started();
    let current = session.game.as_ref().unwrap().current_player().unwrap();
    let other = PlayerId((current.0 + 1) % 4);
    let before = session.snapshot_for(other);
    let connection = session.room.players[other.index()].connection.0;
    let rejected = send(
        &mut session,
        connection,
        ClientCommand::Game(GuandanCommand::Act(GuandanAction::Pass).into()),
    );
    assert!(rejected.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::Rejected {
            reason: RejectReason::Game(GameViolation::Guandan(GuandanViolation::NotYourTurn))
        }
    )));
    assert_eq!(session.snapshot_for(other).your_hand, before.your_hand);
    assert_eq!(session.snapshot_for(other).current, Some(current));
    for player in (0..4).map(PlayerId) {
        let snapshot = session.snapshot_for(player);
        assert_eq!(snapshot.your_hand.len(), 27);
        assert!(snapshot.players.iter().all(|view| view.hand_len == 27));
    }
}

fn spectator_snapshot(deliveries: &[Delivery]) -> &GuandanSnapshot {
    deliveries
        .iter()
        .find_map(|delivery| match &delivery.message.event {
            ServerEvent::GameSnapshot(GameSnapshot::Guandan(snapshot))
                if delivery.recipient == ConnectionId(5) =>
            {
                Some(snapshot)
            }
            _ => None,
        })
        .expect("spectator receives its selected player view")
}

#[test]
fn spectator_follows_private_hands_without_controlling_the_viewed_player() {
    let mut session = started();
    let deliveries = send(&mut session, 5, join(5));
    let initial = spectator_snapshot(&deliveries);
    assert_eq!(initial.you, PlayerId(0));
    assert_eq!(
        initial.your_hand,
        session.snapshot_for(PlayerId(0)).your_hand
    );
    let deliveries = send(
        &mut session,
        5,
        ClientCommand::SpectatePlayer {
            player: PlayerId(1),
        },
    );
    assert_eq!(
        spectator_snapshot(&deliveries),
        &session.snapshot_for(PlayerId(1))
    );
    let deliveries = send(&mut session, 5, ClientCommand::RequestSnapshot);
    assert_eq!(
        spectator_snapshot(&deliveries),
        &session.snapshot_for(PlayerId(1))
    );
    let before = session.snapshot_for(PlayerId(1));
    let rejected = send(
        &mut session,
        5,
        ClientCommand::Game(GuandanCommand::SetAutoPlay { enabled: true }.into()),
    );
    assert!(
        rejected
            .iter()
            .any(|delivery| matches!(delivery.message.event, ServerEvent::Rejected { .. }))
    );
    assert_eq!(session.snapshot_for(PlayerId(1)), before);
    // A raw observer ID still has no private hand; only the selected view is remapped.
    assert!(session.snapshot_for(PlayerId(128)).your_hand.is_empty());
}

#[test]
fn played_cards_are_public_in_every_recipients_snapshot() {
    let mut session = started();
    let current = session.game.as_ref().unwrap().current_player().unwrap();
    let card = session
        .game
        .as_ref()
        .unwrap()
        .player(current)
        .unwrap()
        .hand()[0];
    let connection = session.room.players[current.index()].connection.0;
    let deliveries = send(
        &mut session,
        connection,
        ClientCommand::Game(GuandanCommand::Act(GuandanAction::Play(vec![card])).into()),
    );
    assert!(
        deliveries
            .iter()
            .all(|delivery| !matches!(delivery.message.event, ServerEvent::Rejected { .. }))
    );
    for player in (0..4).map(PlayerId) {
        let snapshot = session.snapshot_for(player);
        assert_eq!(snapshot.played_cards, vec![card]);
    }
    let deliveries = send(&mut session, 5, join(5));
    let spectator = deliveries
        .iter()
        .find_map(|delivery| match &delivery.message.event {
            ServerEvent::GameSnapshot(GameSnapshot::Guandan(snapshot))
                if delivery.recipient == ConnectionId(5) =>
            {
                Some(snapshot)
            }
            _ => None,
        })
        .expect("spectator receives the selected game view");
    assert_eq!(spectator.played_cards, vec![card]);
    assert_eq!(
        spectator.your_hand,
        session.snapshot_for(spectator.you).your_hand
    );
}

#[test]
fn completed_hand_continues_with_tribute_and_preserves_match_id() {
    let mut session = started();
    session.match_state = Some(GuandanMatchState::new(2, PlayerId(0)).unwrap());
    session.game =
        Some(GuandanGameState::new(GuandanRuleSet::default(), build_deck(), PlayerId(0)).unwrap());
    let match_id = session.snapshot_for(PlayerId(0)).match_id;
    for _ in 0..500 {
        let game = session.game.as_ref().unwrap();
        if matches!(game.phase(), GuandanPhase::Finished(_)) {
            break;
        }
        let player = game.current_player().unwrap();
        let action = auto_play_action(game, player).unwrap();
        session.apply_action(player, action, None);
    }
    assert!(matches!(
        session.game.as_ref().unwrap().phase(),
        GuandanPhase::Finished(_)
    ));
    for connection in 1..=4 {
        send(&mut session, connection, ClientCommand::PlayAgain);
    }
    let snapshot = session.snapshot_for(PlayerId(0));
    assert_eq!(snapshot.match_id, match_id);
    assert_eq!(snapshot.hand_number, 2);
    assert!(!snapshot.tribute_pairs.is_empty());
    assert!(matches!(
        snapshot.phase,
        GuandanPhase::Tribute | GuandanPhase::Playing
    ));
    assert!(snapshot.players.iter().all(|player| player.hand_len == 27));
}

#[test]
fn tribute_card_is_visible_only_to_receiver() {
    let mut session = started();
    let result = GuandanGameResult {
        finishing_order: [PlayerId(0), PlayerId(1), PlayerId(2), PlayerId(3)],
        known_positions: 4,
        winning_team: 0,
        level_gain: 2,
    };
    session.game = Some(
        GuandanGameState::new_with_tribute(
            GuandanRuleSet::default(),
            build_deck(),
            PlayerId(0),
            Some(result),
        )
        .unwrap(),
    );
    let card = session
        .game
        .as_ref()
        .unwrap()
        .player(PlayerId(3))
        .unwrap()
        .hand()
        .iter()
        .copied()
        .find(|card| card.is_big_joker())
        .unwrap();
    let deliveries = session.apply_action(PlayerId(3), GuandanAction::Tribute(card), None);
    assert!(deliveries.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::GameEvent(GameEvent::Guandan(GuandanEvent::TributeSubmitted {
            player: PlayerId(3)
        }))
    )));
    for player in (0..4).map(PlayerId) {
        let snapshot = session.snapshot_for(player);
        assert_eq!(
            snapshot.received_tribute,
            (player == PlayerId(0)).then_some(card)
        );
    }
    assert_eq!(session.snapshot_for(PlayerId(128)).received_tribute, None);
}

#[test]
fn complete_match_merges_all_hands_and_unknown_places_once() {
    let mut session = started();
    let prior = GuandanProfileStats {
        completed_games: 2,
        hands_played: 5,
        ..Default::default()
    };
    session.room.players[0].game_profiles.guandan = Some(prior.clone());
    session.match_state = Some(GuandanMatchState::new(13, PlayerId(0)).unwrap());
    let double = GuandanGameResult {
        known_positions: 2,
        finishing_order: [PlayerId(0), PlayerId(2), PlayerId(1), PlayerId(3)],
        winning_team: 0,
        level_gain: 3,
    };
    let state = session.match_state.as_mut().unwrap();
    assert!(state.finish_hand(&double));
    session
        .statistics
        .finish_hand(&double, state.level_advance());
    assert!(state.next_hand());
    assert!(state.finish_hand(&double));
    session
        .statistics
        .finish_hand(&double, state.level_advance());
    let cards = (3..=7)
        .map(|rank| GuandanCard {
            deck: 0,
            suit: GuandanSuit::Spades,
            rank,
        })
        .collect::<Vec<_>>();
    let play = classify(&cards, 14).unwrap();
    session.statistics.observe_play(PlayerId(0), &play);
    session.settle_match();
    let saved = session.room.players[0]
        .game_profiles
        .guandan
        .clone()
        .unwrap();
    assert_eq!(saved.completed_games, prior.completed_games + 1);
    assert_eq!(saved.hands_played, prior.hands_played + 2);
    assert_eq!(saved.hands_won, 2);
    assert_eq!(saved.match_wins, 1);
    assert_eq!(saved.placement_counts, [2, 0, 0, 0]);
    assert_eq!(saved.levels_advanced, 1);
    assert_eq!(saved.double_finish_hands, 2);
    assert_eq!(saved.straight_flush_plays, 1);
    let loser = session.room.players[1]
        .game_profiles
        .guandan
        .as_ref()
        .unwrap();
    assert_eq!(loser.placement_counts, [0; 4]);
    assert_eq!(loser.unranked_hands, 2);
    let completed_games = session.room.players[0].completed_games;
    session.settle_match();
    assert_eq!(
        session.room.players[0].game_profiles.guandan.as_ref(),
        Some(&saved)
    );
    assert_eq!(session.room.players[0].completed_games, completed_games);
}
