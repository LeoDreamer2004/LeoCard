use super::McGuessSession;
use crate::Delivery;
use leocard_protocol::{GameEvent, McGuessEvent, ServerEvent};

impl McGuessSession {
    pub(super) fn analyze_action(&mut self) -> Vec<McGuessEvent> {
        let game = self.game.as_mut().expect("active game");
        let mut events: Vec<_> = game
            .take_action_statistics()
            .into_iter()
            .map(|statistics| McGuessEvent::ActionAnalyzed {
                player: statistics.player,
                statistics: Box::new(statistics),
            })
            .collect();
        let hands = if game.finished() {
            self.match_state
                .as_mut()
                .expect("active match")
                .finish_round(game)
        } else {
            game.take_hand_statistics()
        };
        events.extend(
            hands
                .into_iter()
                .map(|statistics| McGuessEvent::HandAnalyzed {
                    player: statistics.player,
                    statistics,
                }),
        );
        events
    }

    pub(super) fn deliver_game_events(&self, events: Vec<McGuessEvent>) -> Vec<Delivery> {
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
                ServerEvent::GameEvent(GameEvent::McGuess(event)) => {
                    let player = match event {
                        McGuessEvent::ActionAnalyzed { player, .. }
                        | McGuessEvent::HandAnalyzed { player, .. } => *player,
                        McGuessEvent::ProcessPresenceChanged { .. } => return true,
                    };
                    self.room.player_id(delivery.recipient) == Some(player)
                }
                _ => true,
            })
            .collect()
    }
}
