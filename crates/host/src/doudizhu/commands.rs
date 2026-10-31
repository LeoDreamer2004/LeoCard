use super::DoudizhuSession;
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery};
use leocard_doudizhu::achievements::DoudizhuActionContext;
use leocard_doudizhu::{DoudizhuAction, DoudizhuPhase};
use leocard_game_common::PlayerId;
use leocard_protocol::{DoudizhuEvent, GameEvent, RejectReason, RequestId, ServerEvent};

impl DoudizhuSession {
    pub(super) fn apply_action(
        &mut self,
        player: PlayerId,
        action: DoudizhuAction,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        let game = self.game.as_mut().expect("action requires a game");
        let before = DoudizhuActionContext::capture(game);
        if let Err(error) = game.act(player, action.clone()) {
            return origin.map_or_else(Vec::new, |(connection, request)| {
                self.room
                    .reject(connection, request, RejectReason::Game(error.into()))
            });
        }
        self.statistics.observe(player, &action, game);
        let achievements = self
            .achievements
            .observe_action(player, &action, before, game);
        let mut events = vec![DoudizhuEvent::ActionApplied { player, action }];
        events.extend(achievements.into_iter().map(|(player, statistics)| {
            DoudizhuEvent::ActionAnalyzed {
                player,
                statistics: Box::new(statistics),
            }
        }));
        if game.phase == DoudizhuPhase::Redeal {
            self.redeal_hand();
            events.push(DoudizhuEvent::Redealt);
        }
        if let Some(result) = self.game.as_ref().and_then(|g| g.result.clone()) {
            events.push(DoudizhuEvent::HandFinished { result });
            events.extend(self.finish_hand());
        }
        self.reset_automatic_delay();
        self.room.bump_revision();
        let hand_index = u32::from(
            self.match_state
                .as_ref()
                .expect("active match")
                .progress()
                .hand_number,
        );
        let mut deliveries: Vec<_> =
            self.room
                .broadcast_game_events(events, self.match_id.map(|id| (id, Some(hand_index))))
                .into_iter()
                .filter(|delivery| match &delivery.message.event {
                    ServerEvent::GameEvent(GameEvent::Doudizhu(
                        DoudizhuEvent::ActionAnalyzed { player, .. },
                    )) => self.room.player_id(delivery.recipient) == Some(*player),
                    _ => true,
                })
                .collect();
        deliveries.extend(self.broadcast_game(origin));
        deliveries
    }
}
