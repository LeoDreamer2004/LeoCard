use super::DoudizhuSession;
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery};
use leocard_doudizhu::{Action, Phase};
use leocard_protocol::{
    DoudizhuEvent, GameViolation, PlayerId, RejectReason, RequestId, ServerEvent,
};

impl DoudizhuSession {
    pub(super) fn apply_action(
        &mut self,
        player: u8,
        action: Action,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        let game = self.game.as_mut().expect("action requires a game");
        if let Err(error) = game.act(player, action.clone()) {
            return origin.map_or_else(Vec::new, |(connection, request)| {
                self.room.reject(
                    connection,
                    request,
                    RejectReason::Game(GameViolation::Doudizhu(error)),
                )
            });
        }
        let mut events = vec![DoudizhuEvent::ActionApplied {
            player: PlayerId(player),
            action,
        }];
        if game.phase == Phase::Redeal {
            self.redeal_hand();
            events.push(DoudizhuEvent::Redealt);
        }
        if let Some(result) = self.game.as_ref().and_then(|g| g.result.clone()) {
            self.finish_hand();
            events.push(DoudizhuEvent::HandFinished { result });
        }
        self.reset_automatic_delay();
        self.room.bump_revision();
        let mut deliveries = Vec::new();
        for (index, event) in events.into_iter().enumerate() {
            let mut broadcast = self.with_game_context(
                self.room
                    .broadcast_event(None, ServerEvent::GameEvent(event.into())),
            );
            for delivery in &mut broadcast {
                if let Some(context) = &mut delivery.message.game_context {
                    context.sequence += index as u128;
                    context.hand_index = Some(u32::from(
                        self.match_state
                            .as_ref()
                            .expect("active match")
                            .progress()
                            .hand_number,
                    ));
                }
            }
            deliveries.extend(broadcast);
        }
        deliveries.extend(self.broadcast_game(origin));
        deliveries
    }
}
