use super::fixtures::*;
use leocard_protocol::{
    ClientCommand, GameCommand, GameEvent, ServerEvent, TexasHoldemCommand, TexasHoldemEvent,
};
use leocard_texas_holdem::{Phase, TexasHoldemAction};

#[test]
fn settlement_statistics_are_private_and_use_the_same_live_event_context() {
    let mut session = started_session();
    let mut deliveries = Vec::new();
    for request in 10..12 {
        let player = session.game().unwrap().current_player().unwrap();
        let connection = connection_for(&session, player);
        deliveries = session.handle(
            connection,
            message(
                request,
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
    let summaries = deliveries
        .iter()
        .filter_map(|delivery| match &delivery.message.event {
            ServerEvent::GameEvent(GameEvent::TexasHoldem(TexasHoldemEvent::HandAnalyzed {
                player,
                statistics,
            })) => Some((delivery, player, statistics)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(summaries.len(), 3);
    for (delivery, player, statistics) in summaries {
        assert_eq!(delivery.recipient, connection_for(&session, *player));
        let snapshot = session.game().unwrap().snapshot(*player).unwrap();
        assert_eq!(statistics.hole_cards, snapshot.your_hole_cards);
        let context = delivery.message.game_context.unwrap();
        assert_eq!(context.match_id, snapshot.match_id);
        assert_eq!(context.hand_index, Some(snapshot.hand_number));
        let finish = deliveries
            .iter()
            .find(|candidate| {
                candidate.recipient == delivery.recipient
                    && matches!(
                        candidate.message.event,
                        ServerEvent::GameEvent(GameEvent::TexasHoldem(
                            TexasHoldemEvent::HandFinished { .. }
                        ))
                    )
            })
            .unwrap();
        assert!(context.sequence < finish.message.game_context.unwrap().sequence);
    }
}
