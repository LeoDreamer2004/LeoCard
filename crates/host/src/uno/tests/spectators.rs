use super::super::UnoSession;
use super::*;
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery};
use leocard_game_common::PlayerId;
use leocard_protocol::{ClientCommand, GameSnapshot, SeatId, ServerEvent, UnoSnapshot};
use leocard_uno::{UnoColor, UnoFace, UnoMode, UnoRuleSet, build_no_mercy_deck};

fn snapshot(deliveries: &[Delivery], recipient: ConnectionId) -> &UnoSnapshot {
    deliveries
        .iter()
        .find_map(|delivery| match &delivery.message.event {
            ServerEvent::GameSnapshot(GameSnapshot::Uno(game))
                if delivery.recipient == recipient =>
            {
                Some(game)
            }
            _ => None,
        })
        .expect("recipient receives its selected private view")
}

#[test]
fn only_eliminated_no_mercy_players_can_switch_private_views() {
    let rules = UnoRuleSet {
        mode: UnoMode::NoMercy,
        ..Default::default()
    };
    // The first draw runs to 25 cards without finding a playable red card or five.
    let (mut misses, rest): (Vec<_>, Vec<_>) =
        build_no_mercy_deck().into_iter().partition(|card| {
            card.color() != Some(UnoColor::Red)
                && matches!(card.face(), UnoFace::Number(number) if number != 5)
        });
    misses.extend(rest);
    let top = misses
        .iter()
        .position(|card| card.color() == Some(UnoColor::Red) && card.face() == UnoFace::Number(5))
        .unwrap();
    let top = misses.remove(top);
    misses.insert(21, top);
    let mut session = UnoSession::new(ROOM, 52300, rules, misses).unwrap();
    for index in 0..3 {
        let connection = ConnectionId(index + 1);
        session.handle(connection, message(1, join_command("玩家", index + 1)));
        session.room.players[index as usize].seat = Some(SeatId(index as u8));
        session.handle(
            connection,
            message(2, ClientCommand::SetReady { ready: true }),
        );
    }
    session.handle(HOST, message(3, ClientCommand::StartGame));
    let target = PlayerId(1);
    let rejected = session.handle(
        HOST,
        message(4, ClientCommand::SpectatePlayer { player: target }),
    );
    assert!(
        rejected
            .iter()
            .any(|delivery| matches!(delivery.message.event, ServerEvent::Rejected { .. }))
    );
    session
        .game
        .as_mut()
        .unwrap()
        .draw_card(PlayerId(0))
        .unwrap();
    assert!(
        session
            .game()
            .unwrap()
            .player(PlayerId(0))
            .unwrap()
            .eliminated()
    );
    let expected = session.game_snapshot(target).your_hand;
    let deliveries = session.handle(
        HOST,
        message(5, ClientCommand::SpectatePlayer { player: target }),
    );
    assert_eq!(snapshot(&deliveries, HOST).you, target);
    assert_eq!(snapshot(&deliveries, HOST).your_hand, expected);
    let broadcast = session.broadcast_game(None);
    assert_eq!(snapshot(&broadcast, HOST).your_hand, expected);
    let requested = session.handle(HOST, message(6, ClientCommand::RequestSnapshot));
    assert_eq!(snapshot(&requested, HOST).you, target);
    // Commands still use the eliminated participant's connection identity.
    assert_eq!(session.room.player_id(HOST), Some(PlayerId(0)));
    let other = session.handle(
        ConnectionId(3),
        message(7, ClientCommand::SpectatePlayer { player: target }),
    );
    assert!(
        other
            .iter()
            .any(|delivery| matches!(delivery.message.event, ServerEvent::Rejected { .. }))
    );
    session.start_next_game(None);
    let restarted = session.handle(HOST, message(8, ClientCommand::RequestSnapshot));
    assert_eq!(snapshot(&restarted, HOST).you, PlayerId(0));
    assert!(session.eliminated_views.is_empty());
}
