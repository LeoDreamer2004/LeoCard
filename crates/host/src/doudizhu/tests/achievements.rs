use super::{act, assert_accepted, join, send, started};
use crate::{ConnectionId, DoudizhuSession};
use leocard_doudizhu::achievements::DoudizhuAchievementFacts;
use leocard_doudizhu::{Action, Card, Mode, Phase, RuleSet, bot_action, build_deck};
use leocard_protocol::{
    ClientCommand, DoudizhuEvent, GameEvent, PlayerId, PlayerInteractionKind, RoomId, SeatId,
    ServerEvent,
};
use std::collections::HashMap;

#[test]
fn live_analysis_is_private_sequenced_and_only_released_at_the_right_stage() {
    for mode in [Mode::Three, Mode::Four] {
        let mut session = started(mode, false);
        let first = session.game.as_ref().unwrap().current;
        let deliveries = act(&mut session, first, Action::Bid(3));
        assert!(deliveries.iter().all(|delivery| !matches!(
            delivery.message.event,
            ServerEvent::GameEvent(GameEvent::Doudizhu(DoudizhuEvent::ActionAnalyzed { .. }))
        )));
        let mut observed = HashMap::new();
        let mut hand_reports = 0;
        for _ in 0..1500 {
            if session.game.as_ref().unwrap().phase == Phase::Finished {
                break;
            }
            let game = session.game.as_ref().unwrap();
            let player = game.current;
            let action = bot_action(game).unwrap();
            for delivery in act(&mut session, player, action) {
                let ServerEvent::GameEvent(GameEvent::Doudizhu(DoudizhuEvent::ActionAnalyzed {
                    player,
                    statistics,
                })) = &delivery.message.event
                else {
                    continue;
                };
                assert_eq!(session.room.player_id(delivery.recipient), Some(*player));
                let context = delivery.message.game_context.unwrap();
                assert_eq!(Some(context.match_id), session.match_id);
                assert_eq!(context.hand_index, Some(1));
                assert!(context.records_progress);
                if let Some(previous) = observed.insert(*player, context.sequence) {
                    assert!(context.sequence > previous);
                }
                if matches!(**statistics, DoudizhuAchievementFacts::Hand(_)) {
                    assert_eq!(session.game.as_ref().unwrap().phase, Phase::Finished);
                    hand_reports += 1;
                }
            }
        }
        assert_eq!(session.game.as_ref().unwrap().phase, Phase::Finished);
        assert_eq!(hand_reports, mode.players());
        assert_eq!(observed.len(), usize::from(mode.players()));
    }
}

#[test]
fn a_validated_flower_interaction_reaches_the_game_tracker_and_awards_its_sender() {
    let reserved: Vec<_> = (4..8).map(Card).collect();
    let mut rest = build_deck(Mode::Three)
        .into_iter()
        .filter(|card| !reserved.contains(card));
    let mut deck = Vec::new();
    for index in 0..51 {
        deck.push(if index % 3 == 1 && index / 3 < 4 {
            reserved[index / 3]
        } else {
            rest.next().unwrap()
        });
    }
    deck.extend(rest);
    let mut session = DoudizhuSession::new(RoomId(99), 52300, RuleSet::default(), deck).unwrap();
    for connection in 1..=3 {
        assert_accepted(&send(&mut session, connection, join(connection)));
        assert_accepted(&send(
            &mut session,
            connection,
            ClientCommand::SelectSeat {
                seat: SeatId(connection as u8 - 1),
            },
        ));
        assert_accepted(&send(
            &mut session,
            connection,
            ClientCommand::SetReady { ready: true },
        ));
    }
    assert_accepted(&send(&mut session, 1, ClientCommand::StartGame));
    while session.game.as_ref().unwrap().current != 0 {
        let player = session.game.as_ref().unwrap().current;
        assert_accepted(&act(&mut session, player, Action::Bid(0)));
    }
    assert_accepted(&act(&mut session, 0, Action::Bid(3)));
    let card = session.game.as_ref().unwrap().hand(0).unwrap()[0];
    assert_accepted(&act(&mut session, 0, Action::Play(vec![card])));
    assert_accepted(&send(
        &mut session,
        3,
        ClientCommand::Interact {
            target: PlayerId(1),
            kind: PlayerInteractionKind::Flower,
        },
    ));
    let deliveries = act(&mut session, 1, Action::Play(reserved));
    assert_accepted(&deliveries);
    let flowers: Vec<_> = deliveries.iter().filter(|delivery| matches!(&delivery.message.event, ServerEvent::GameEvent(GameEvent::Doudizhu(DoudizhuEvent::ActionAnalyzed { player: PlayerId(2), statistics })) if **statistics == DoudizhuAchievementFacts::FlowerBomb)).collect();
    assert_eq!(flowers.len(), 1);
    assert_eq!(flowers[0].recipient, ConnectionId(3));
}
