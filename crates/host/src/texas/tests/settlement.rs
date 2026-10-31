use super::super::*;
use super::*;
use crate::ConnectionId;
use leocard_protocol::{
    ClientCommand, GameCommand, GameSnapshot, ServerEvent, TexasCommand, TexasPhaseView,
};
use leocard_texas_holdem::{
    TexasAction, TexasCard, TexasPhase, TexasRank, TexasRuleSet, TexasSuit, build_deck,
};
use std::collections::HashMap;

#[test]
fn all_players_ready_start_the_next_hand_with_stacks_and_rotated_dealer_preserved() {
    let mut session = started_session();
    let first_dealer = session.game().unwrap().game().dealer();
    let mut requests = HashMap::from([
        (HOST_CONNECTION, 2_u64),
        (SECOND_CONNECTION, 2_u64),
        (THIRD_CONNECTION, 2_u64),
    ]);
    while matches!(
        session.game().unwrap().game().phase(),
        TexasPhase::Betting(_)
    ) {
        let player = session.game().unwrap().current_player().unwrap();
        let connection = connection_for(&session, player);
        let request = requests.get_mut(&connection).unwrap();
        *request += 1;
        session.handle(
            connection,
            message(
                *request,
                ClientCommand::Game(GameCommand::Texas(TexasCommand::Act {
                    action: TexasAction::Fold,
                })),
            ),
        );
    }
    assert!(matches!(
        session.game().unwrap().game().phase(),
        TexasPhase::Complete(_)
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
                TexasPhaseView::HandComplete { .. }
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
    assert!(matches!(game.phase(), TexasPhase::Betting(_)));
}

#[test]
fn first_busted_player_finishes_the_tournament_and_applies_shared_rating_once() {
    let prefix = [
        TexasCard::new(TexasSuit::Spade, TexasRank::King),
        TexasCard::new(TexasSuit::Spade, TexasRank::Queen),
        TexasCard::new(TexasSuit::Spade, TexasRank::Ace),
        TexasCard::new(TexasSuit::Heart, TexasRank::King),
        TexasCard::new(TexasSuit::Heart, TexasRank::Queen),
        TexasCard::new(TexasSuit::Heart, TexasRank::Ace),
        TexasCard::new(TexasSuit::Club, TexasRank::Two),
        TexasCard::new(TexasSuit::Diamond, TexasRank::Three),
        TexasCard::new(TexasSuit::Spade, TexasRank::Seven),
        TexasCard::new(TexasSuit::Club, TexasRank::Eight),
        TexasCard::new(TexasSuit::Diamond, TexasRank::Nine),
    ];
    let deck = prefix
        .into_iter()
        .chain(
            build_deck(false)
                .into_iter()
                .filter(|card| !prefix.contains(card)),
        )
        .collect();
    let mut session = TexasSession::new(
        ROOM,
        52300,
        TexasRuleSet {
            starting_chips: 5,
            ..TexasRuleSet::default()
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
                ClientCommand::Game(GameCommand::Texas(TexasCommand::Act {
                    action: TexasAction::PostBlind,
                })),
            ),
        );
        request += 1;
    }
    while matches!(
        session.game().unwrap().game().phase(),
        TexasPhase::Betting(_)
    ) {
        let player = session.game().unwrap().current_player().unwrap();
        let connection = connection_for(&session, player);
        session.handle(
            connection,
            message(
                request,
                ClientCommand::Game(GameCommand::Texas(TexasCommand::Act {
                    action: TexasAction::AllIn,
                })),
            ),
        );
        request += 1;
    }
    let snapshot = session.game_snapshot(session.room.players[0].id);
    let TexasPhaseView::HandComplete {
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
        let stats = player.player.game_profiles.texas.as_ref().unwrap();
        let delta = reference_changes
            .iter()
            .find(|change| change.player == player.player.id)
            .unwrap()
            .delta;
        // This table starts with five chips each: its sole winner receives all fifteen.
        // Comparing against the default twenty would incorrectly erase that winner's award.
        assert_eq!(delta, if player.stack == 15 { 3 } else { 0 });
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

    let host_id = session.room.player_id(HOST_CONNECTION).unwrap();
    let host_seat = session.room.players[usize::from(host_id.0)].seat;
    session.disconnect(THIRD_CONNECTION);
    let deliveries = session.handle(
        HOST_CONNECTION,
        message(request, ClientCommand::ReturnToLobby),
    );
    let lobby = deliveries
        .iter()
        .find_map(|delivery| match &delivery.message.event {
            ServerEvent::LobbySnapshot(lobby) => Some(lobby),
            _ => None,
        })
        .expect("finished tournament returns to the lobby");
    assert_eq!(
        lobby.players.len(),
        2,
        "offline players must release their slots"
    );
    assert_eq!(lobby.host, Some(host_id));
    assert!(lobby.players.iter().all(|player| player.seat.is_some()));
    assert_eq!(
        lobby
            .players
            .iter()
            .find(|player| player.player.id == host_id)
            .unwrap()
            .seat,
        host_seat
    );
    let replacement = ConnectionId(40);
    session.handle(replacement, message(request + 1, join_command("新玩家", 4)));
    for connection in [SECOND_CONNECTION, replacement] {
        session.handle(
            connection,
            message(request + 2, ClientCommand::SetReady { ready: true }),
        );
    }
    let restarted = session.handle(
        HOST_CONNECTION,
        message(request + 3, ClientCommand::StartGame),
    );
    assert!(
        restarted.iter().any(|delivery| matches!(
            delivery.message.event,
            ServerEvent::GameSnapshot(GameSnapshot::Texas(_))
        )),
        "{restarted:?}"
    );
}
