use super::MinecraftGuessSession;
use crate::Delivery;
use leocard_protocol::{GameEvent, MinecraftGuessEvent, PlayerId, ServerEvent};

impl MinecraftGuessSession {
    pub(super) fn analyze_action(&mut self) -> Vec<MinecraftGuessEvent> {
        let game = self.game.as_mut().expect("active game");
        let mut events: Vec<_> = game
            .take_action_statistics()
            .into_iter()
            .map(|statistics| MinecraftGuessEvent::ActionAnalyzed {
                player: PlayerId(statistics.player),
                statistics: Box::new(statistics),
            })
            .collect();
        if game.finished() {
            events.extend(
                self.match_state
                    .as_mut()
                    .expect("active match")
                    .finish_round(game)
                    .into_iter()
                    .map(|statistics| MinecraftGuessEvent::HandAnalyzed {
                        player: PlayerId(statistics.player),
                        statistics,
                    }),
            );
        }
        events
    }

    pub(super) fn deliver_game_events(&self, events: Vec<MinecraftGuessEvent>) -> Vec<Delivery> {
        let hand = u32::from(
            self.match_state
                .as_ref()
                .expect("active match")
                .progress()
                .game_index,
        );
        self.room
            .broadcast_game_events(events, self.match_id.map(|id| (id, Some(hand))))
            .into_iter()
            .filter(|delivery| match &delivery.message.event {
                ServerEvent::GameEvent(GameEvent::MinecraftGuess(event)) => {
                    let player = match event {
                        MinecraftGuessEvent::ActionAnalyzed { player, .. }
                        | MinecraftGuessEvent::HandAnalyzed { player, .. } => *player,
                        MinecraftGuessEvent::ProcessPresenceChanged { .. } => return true,
                    };
                    self.room.player_id(delivery.recipient) == Some(player)
                }
                _ => true,
            })
            .collect()
    }
}
