use super::DoudizhuSession;
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery, new_match_id};
use leocard_doudizhu::{Action, GameState, Phase, build_deck};
use leocard_protocol::{
    DoudizhuEvent, GameViolation, PlayerId, RejectReason, RequestId, ServerEvent,
};

impl DoudizhuSession {
    pub(super) fn start_round(&mut self) {
        let mut deck = self
            .deck
            .take()
            .unwrap_or_else(|| build_deck(self.rules.mode));
        if self.match_id.is_some() {
            fastrand::shuffle(&mut deck);
        }
        self.game = Some(
            GameState::new(self.rules, deck, fastrand::u8(..self.rules.mode.players()))
                .expect("validated deck and rules"),
        );
        self.match_id = Some(new_match_id());
        for player in &mut self.room.players {
            player.ready = player.is_bot;
            player.auto_play = false;
        }
        self.reset_automatic_delay();
    }

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
            let mut deck = build_deck(self.rules.mode);
            fastrand::shuffle(&mut deck);
            self.game = Some(
                GameState::new(self.rules, deck, (player + 1) % self.rules.mode.players())
                    .expect("fresh deck"),
            );
            events.push(DoudizhuEvent::Redealt);
        }
        if let Some(result) = self.game.as_ref().and_then(|g| g.result.clone()) {
            self.room.prepare_rematch();
            events.push(DoudizhuEvent::Finished { result });
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
                }
            }
            deliveries.extend(broadcast);
        }
        deliveries.extend(self.broadcast_game(origin));
        deliveries
    }
}
