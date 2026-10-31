use super::super::McGuessSession;
use super::support::{join, send, snapshot};
use crate::ConnectionId;
use ed25519_dalek::{Signer, SigningKey};
use leocard_game_common::PlayerId;
use leocard_mc_guess::{McGuessCatalogue, McGuessEntryKind, McGuessGameState, McGuessRuleSet};
use leocard_protocol::{
    ClientCommand, JoinRequest, McGuessCommand, PlayerGameProfiles, ProfileId, ReconnectToken,
    RejectReason, RoomId, RoomViolation, SeatId, ServerEvent,
};

#[test]
fn eight_players_take_every_first_turn_and_record_all_eight_placements() {
    let rules = McGuessRuleSet::default();
    let mut session = McGuessSession::new(RoomId(99), 52300, rules);
    for connection in 1..=8 {
        join(&mut session, connection);
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
    let key = SigningKey::from_bytes(&[9; 32]);
    let mut request = JoinRequest {
        name: "第九人".to_owned(),
        reconnect_token: ReconnectToken(9),
        profile_id: ProfileId(key.verifying_key().to_bytes()),
        reference_points: 0,
        completed_games: 0,
        game_profiles: PlayerGameProfiles::default(),
        identity_signature: Vec::new(),
    };
    request.identity_signature = key
        .sign(&request.identity_payload(RoomId(99)))
        .to_bytes()
        .to_vec();
    let rejected = send(&mut session, 9, ClientCommand::join(request));
    assert!(matches!(
        rejected[0].message.event,
        ServerEvent::Rejected {
            reason: RejectReason::Room(RoomViolation::RoomFull),
        }
    ));
    let started = send(&mut session, 1, ClientCommand::StartGame);
    assert_eq!(snapshot(&started, 8).progress.total_games(), 8);
    // Unequal prior gains exercise the last-place archive slot as well as the winner's.
    session
        .match_state
        .as_mut()
        .unwrap()
        .record_gains((0..8).map(|id| (PlayerId(id), u32::from(id) * 20)));
    let target = McGuessCatalogue::bundled()
        .find(McGuessEntryKind::Block, "minecraft:stone")
        .unwrap()
        .clone();
    for game_index in 0..8 {
        let order = session.match_state.as_ref().unwrap().turn_order();
        assert_eq!(order[0], PlayerId(game_index));
        session.game = Some(McGuessGameState::new(target.clone(), order, rules));
        let finished = send(
            &mut session,
            u64::from(game_index) + 1,
            ClientCommand::Game(
                McGuessCommand::Guess {
                    entry: target.id.clone(),
                }
                .into(),
            ),
        );
        let view = snapshot(&finished, 8);
        assert_eq!(view.round.winner, Some(PlayerId(game_index)));
        if game_index == 7 {
            assert_eq!(
                view.reference_changes
                    .iter()
                    .map(|change| change.delta)
                    .collect::<Vec<_>>(),
                vec![-5, -3, -2, -1, 1, 2, 3, 5]
            );
            for (index, player) in view.players.iter().enumerate() {
                let stats = player.player.game_profiles.mc_guess.as_ref().unwrap();
                assert_eq!(stats.completed_games, 1);
                assert_eq!(stats.categories.block.games, 8);
                assert_eq!(stats.categories.block.wins, 1);
                assert_eq!(stats.placement_counts[7 - index], 1);
            }
        } else {
            for connection in 1..=8 {
                send(&mut session, connection, ClientCommand::PlayAgain);
            }
        }
    }
    assert!(session.is_current_connection(ConnectionId(8)));
}
