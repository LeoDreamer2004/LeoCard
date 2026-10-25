use super::super::TexasHoldemSession;
use super::fixtures::{
    HOST_CONNECTION, ROOM, SECOND_CONNECTION, THIRD_CONNECTION, connection_for, join_command,
    message,
};
use leocard_protocol::{
    ClientCommand, GameEvent, GameViolation, RejectReason, ServerEvent, ServerMessage,
    TexasHoldemCommand, TexasHoldemEvent, decode_frame, encode_frame,
};
use leocard_texas_holdem::{
    TexasHoldemAction, TexasHoldemRuleSet, build_deck, deepseek_squid_marker,
};

#[test]
fn lobby_enforces_the_current_player_count_instead_of_table_capacity() {
    let mut session = TexasHoldemSession::new(
        ROOM,
        52300,
        TexasHoldemRuleSet::default(),
        build_deck(false),
    )
    .unwrap();
    for (connection, token) in [
        (HOST_CONNECTION, 1),
        (SECOND_CONNECTION, 2),
        (THIRD_CONNECTION, 3),
    ] {
        session.handle(connection, message(1, join_command("玩家", token)));
    }
    for (request, count) in [(2, 3), (3, 2)] {
        let rules = TexasHoldemRuleSet {
            squid_markers: count,
            ..Default::default()
        };
        let deliveries = session.handle(
            HOST_CONNECTION,
            message(
                request,
                ClientCommand::Game(TexasHoldemCommand::UpdateRules { rules }.into()),
            ),
        );
        assert_eq!(
            deliveries.iter().any(|delivery| matches!(
                delivery.message.event,
                ServerEvent::Rejected {
                    reason: RejectReason::Game(GameViolation::InvalidRuleConfiguration)
                }
            )),
            count == 3
        );
    }
}

#[test]
fn a_squid_award_is_broadcast_once_as_a_live_serializable_fact() {
    let mut session = TexasHoldemSession::new(
        ROOM,
        52300,
        TexasHoldemRuleSet {
            squid_markers: 2,
            ..Default::default()
        },
        build_deck(false),
    )
    .unwrap();
    for (connection, token) in [
        (HOST_CONNECTION, 1),
        (SECOND_CONNECTION, 2),
        (THIRD_CONNECTION, 3),
    ] {
        session.handle(connection, message(1, join_command("玩家", token)));
        session.handle(
            connection,
            message(2, ClientCommand::SetReady { ready: true }),
        );
    }
    session.handle(HOST_CONNECTION, message(3, ClientCommand::StartGame));
    let mut deliveries = Vec::new();
    for (request, action) in [
        (10, TexasHoldemAction::PostBlind),
        (11, TexasHoldemAction::PostBlind),
        (12, TexasHoldemAction::Fold),
        (13, TexasHoldemAction::Fold),
    ] {
        let player = session.game().unwrap().current_player().unwrap();
        let connection = connection_for(&session, player);
        deliveries = session.handle(
            connection,
            message(
                request,
                ClientCommand::Game(TexasHoldemCommand::Act { action }.into()),
            ),
        );
    }
    let game = session.game().unwrap();
    let awarded = deliveries
        .iter()
        .filter_map(|delivery| match &delivery.message.event {
            ServerEvent::GameEvent(GameEvent::TexasHoldem(TexasHoldemEvent::SquidAwarded {
                player,
                statistics,
            })) => Some((delivery, player, statistics)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(awarded.len(), 3);
    for (delivery, player, statistics) in awarded {
        let snapshot = game.snapshot(*player).unwrap();
        let drawn = snapshot
            .squid
            .markers
            .iter()
            .position(|owner| *owner == Some(*player))
            .unwrap();
        assert_eq!(snapshot.squid.markers.iter().flatten().count(), 1);
        assert_eq!(statistics.owned, 1);
        assert_eq!(
            statistics.deepseek,
            deepseek_squid_marker(snapshot.match_id.0, snapshot.squid.round, drawn).is_some()
        );
        let context = delivery.message.game_context.unwrap();
        assert_eq!(context.match_id, snapshot.match_id);
        assert_eq!(context.hand_index, Some(snapshot.hand_number));
        assert_eq!(
            decode_frame::<ServerMessage>(&encode_frame(&delivery.message).unwrap()).unwrap(),
            delivery.message
        );
    }
    let refreshed = session.handle(HOST_CONNECTION, message(20, ClientCommand::RequestSnapshot));
    assert!(!refreshed.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::GameEvent(GameEvent::TexasHoldem(
            TexasHoldemEvent::SquidAwarded { .. }
        ))
    )));
}
