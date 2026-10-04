use super::*;
use crate::lifecycle::HostedGameLifecycle;
use leocard_protocol::{
    ClientCommand, GameCommand, GameEvent, ServerEvent, ServerMessage, ShengjiCommand,
    ShengjiEvent, decode_frame, encode_frame,
};
use leocard_shengji::Phase;

#[test]
fn hand_analysis_is_private_sequenced_and_absent_before_settlement() {
    let (mut session, connections, target) = started_session();
    session.advance_time(DEAL_INTERVAL);
    session.handle(
        connections[0],
        message(
            0,
            5,
            ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Declare {
                cards: vec![target],
            })),
        ),
    );
    session.advance_time(DEAL_INTERVAL * 99);
    session.advance_time(BIDDING_GRACE);
    assert!(session.hand_analysis_events().is_empty());
    for (index, connection) in connections.iter().copied().enumerate() {
        session.handle(
            connection,
            message(
                index as u8,
                6,
                ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::SetAutoPlay {
                    enabled: true,
                })),
            ),
        );
    }
    let mut received = [0; 4];
    for _ in 0..400 {
        let deliveries = session.advance_time(Duration::from_secs(2));
        for delivery in deliveries {
            if let ServerEvent::GameEvent(GameEvent::Shengji(ShengjiEvent::HandAnalyzed {
                player,
                statistics,
                ..
            })) = &delivery.message.event
            {
                assert!(matches!(
                    session.game().unwrap().phase(),
                    Phase::Finished(_)
                ));
                assert_eq!(session.room.player_id(delivery.recipient), Some(*player));
                assert_eq!(player.0, statistics.player.0);
                assert_eq!(statistics.opening_hand.card_count, 25);
                let context = delivery.message.game_context.unwrap();
                assert_eq!(Some(context.match_id), session.match_id);
                assert_eq!(context.hand_index, Some(session.hand_number));
                let encoded = encode_frame(&delivery.message).unwrap();
                assert_eq!(
                    decode_frame::<ServerMessage>(&encoded).unwrap(),
                    delivery.message
                );
                received[usize::from(player.0)] += 1;
            }
        }
        if matches!(session.game().unwrap().phase(), Phase::Finished(_)) {
            break;
        }
    }
    assert_eq!(received, [1; 4]);
    assert!(
        session
            .room
            .players
            .iter()
            .all(|player| player.completed_games == 1)
    );
    assert!(
        session
            .broadcast_game(None)
            .iter()
            .all(|delivery| delivery.message.game_context.is_none())
    );
    for delivery in session.advance_time(Duration::from_secs(10)) {
        assert!(!matches!(
            delivery.message.event,
            ServerEvent::GameEvent(GameEvent::Shengji(ShengjiEvent::HandAnalyzed { .. }))
        ));
    }
}

#[test]
fn match_shutout_counter_updates_once_per_hand_and_survives_rematch() {
    let (mut session, _, _) = started_session();
    let result = super::settlement::result_for_reference_points(true, 3, 0);
    for count in 1..=3 {
        session.apply_finished_reference_points(&result);
        session.apply_finished_reference_points(&result);
        assert_eq!(
            session
                .statistics
                .match_statistics
                .consecutive_dealer_shutouts,
            [count, 0]
        );
        session.statistics.start_hand();
    }
    session.apply_finished_reference_points(&super::settlement::result_for_reference_points(
        true, 1, 40,
    ));
    assert_eq!(
        session
            .statistics
            .match_statistics
            .consecutive_dealer_shutouts,
        [0, 0]
    );
}
