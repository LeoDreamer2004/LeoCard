use super::{assert_accepted, finish_hand, join, ready_players, send};
use crate::DoudizhuSession;
use leocard_doudizhu::{Mode, RuleSet, build_deck};
use leocard_protocol::{
    ClientCommand, DoudizhuCommand, PlayerId, RejectReason, RoomId, RoomViolation, SeatId,
    ServerEvent,
};
use std::collections::HashSet;

fn lobby(seats: &[u8]) -> DoudizhuSession {
    let mut session = DoudizhuSession::new(
        RoomId(99),
        52300,
        RuleSet::for_mode(Mode::Four),
        build_deck(Mode::Four),
    )
    .unwrap();
    for (index, seat) in seats.iter().enumerate() {
        let connection = index as u64 + 1;
        assert_accepted(&send(&mut session, connection, join(connection)));
        assert_accepted(&send(
            &mut session,
            connection,
            ClientCommand::SelectSeat {
                seat: SeatId(*seat),
            },
        ));
    }
    session
}

fn switch(session: &mut DoudizhuSession, mode: Mode) -> Vec<crate::Delivery> {
    send(
        session,
        1,
        ClientCommand::Game(
            DoudizhuCommand::UpdateRules {
                rules: RuleSet::for_mode(mode),
            }
            .into(),
        ),
    )
}

fn ready_and_start(session: &mut DoudizhuSession, connections: &[u64]) {
    for connection in connections {
        assert_accepted(&send(
            session,
            *connection,
            ClientCommand::SetReady { ready: true },
        ));
    }
    assert_accepted(&send(session, 1, ClientCommand::StartGame));
    assert!(session.game.is_some());
}

#[test]
fn shrinking_with_three_players_reseats_the_fourth_position_and_can_start() {
    for vacant in 0..4 {
        let seats: Vec<_> = (0..4).filter(|seat| *seat != vacant).collect();
        let mut session = lobby(&seats);
        assert_accepted(&switch(&mut session, Mode::Three));
        let actual: HashSet<_> = session
            .room
            .players
            .iter()
            .filter(|player| !player.left)
            .map(|player| player.seat.expect("mode changes must not unseat a player"))
            .collect();
        assert_eq!(actual, (0..3).map(SeatId).collect());
        for (index, seat) in seats.iter().enumerate().filter(|(_, seat)| **seat < 3) {
            assert_eq!(session.room.players[index].seat, Some(SeatId(*seat)));
        }
        ready_and_start(&mut session, &[1, 2, 3]);
        assert_eq!(session.game.as_ref().unwrap().rules.mode, Mode::Three);
    }
}

#[test]
fn departed_players_do_not_block_a_shrink_or_change_remaining_player_ids() {
    let mut session = lobby(&[0, 1, 2, 3]);
    assert_accepted(&send(&mut session, 3, ClientCommand::LeaveRoom));
    assert_accepted(&switch(&mut session, Mode::Three));
    assert_eq!(
        session.room.player_id(crate::ConnectionId(4)),
        Some(PlayerId(3))
    );
    assert_eq!(session.room.players[3].seat, Some(SeatId(2)));
    ready_and_start(&mut session, &[1, 2, 4]);
}

#[test]
fn a_full_four_player_room_rejects_shrinking_without_changing_rules_or_seats() {
    let mut session = lobby(&[0, 1, 2, 3]);
    let before: Vec<_> = session
        .room
        .players
        .iter()
        .map(|player| player.seat)
        .collect();
    let revision = session.revision();
    let deliveries = switch(&mut session, Mode::Three);
    assert!(deliveries.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::Rejected {
            reason: RejectReason::Room(RoomViolation::RoomFull)
        }
    )));
    assert_eq!(session.rules.mode, Mode::Four);
    assert_eq!(session.room.seat_count, 4);
    assert_eq!(session.revision(), revision);
    assert_eq!(
        session
            .room
            .players
            .iter()
            .map(|player| player.seat)
            .collect::<Vec<_>>(),
        before
    );
}

#[test]
fn bots_relocated_during_shrinking_do_not_conflict_with_new_bots_after_expanding() {
    let mut session = lobby(&[0]);
    for seat in [1, 3] {
        assert_accepted(&send(
            &mut session,
            1,
            ClientCommand::ConfigureBotSeat {
                seat: SeatId(seat),
                occupied: true,
            },
        ));
    }
    assert_accepted(&switch(&mut session, Mode::Three));
    assert!(
        session
            .room
            .players
            .iter()
            .all(|player| player.seat.is_some_and(|seat| seat.0 < 3))
    );
    assert_accepted(&switch(&mut session, Mode::Four));
    assert_accepted(&send(
        &mut session,
        1,
        ClientCommand::ConfigureBotSeat {
            seat: SeatId(3),
            occupied: true,
        },
    ));
    assert_eq!(
        session
            .room
            .players
            .iter()
            .map(|player| player.connection)
            .collect::<HashSet<_>>()
            .len(),
        4
    );
    ready_and_start(&mut session, &[1]);
}

#[test]
fn returning_with_bots_keeps_the_hosts_seat_and_game_identity() {
    let mut session = lobby(&[2]);
    assert_accepted(&switch(&mut session, Mode::Three));
    for seat in [0, 1] {
        assert_accepted(&send(
            &mut session,
            1,
            ClientCommand::ConfigureBotSeat {
                seat: SeatId(seat),
                occupied: true,
            },
        ));
    }
    ready_and_start(&mut session, &[1]);
    let host_id = session.room.player_id(crate::ConnectionId(1)).unwrap();
    assert_eq!(host_id, PlayerId(2));
    for hand in 0..Mode::Three.players() {
        finish_hand(&mut session);
        if hand + 1 < Mode::Three.players() {
            ready_players(&mut session);
        }
    }
    let deliveries = send(&mut session, 1, ClientCommand::ReturnToLobby);
    assert_accepted(&deliveries);
    let lobby = deliveries
        .iter()
        .find_map(|delivery| match &delivery.message.event {
            ServerEvent::LobbySnapshot(lobby) => Some(lobby),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        lobby.host,
        Some(host_id),
        "the client still uses its in-game identity"
    );
    assert_eq!(lobby.players.len(), 1);
    assert_eq!(lobby.players[0].id, host_id);
    assert_eq!(lobby.players[0].seat, Some(SeatId(2)));
    assert!(lobby.players[0].ready);
    assert_accepted(&send(&mut session, 2, join(2)));
    assert_eq!(
        session.room.player_id(crate::ConnectionId(1)),
        Some(host_id)
    );
}
