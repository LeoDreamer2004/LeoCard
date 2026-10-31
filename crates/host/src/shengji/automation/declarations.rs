use super::super::ShengjiSession;
use leocard_protocol::ShengjiEvent;
use leocard_shengji::{ShengjiBiddingObservation, ShengjiBot};

impl ShengjiSession {
    pub(in super::super) fn open_bot_declarations(&mut self) -> Vec<ShengjiEvent> {
        let players = self
            .room
            .players
            .iter()
            .filter(|participant| participant.is_bot && !participant.left)
            .map(|participant| participant.id)
            .collect::<Vec<_>>();
        let mut events = Vec::new();
        // Every accepted declaration is stronger, so reactions settle after finitely many bids.
        loop {
            let mut changed = false;
            for &player in &players {
                let cards = self.game.as_ref().and_then(|game| {
                    let observation = ShengjiBiddingObservation::from_game(game, player)?;
                    ShengjiBot::declaration(&observation)
                });
                if let Some(cards) = cards
                    && self
                        .game
                        .as_mut()
                        .is_some_and(|game| game.declare(player, &cards).is_ok())
                {
                    changed = true;
                    self.record_current_declaration();
                    events.push(ShengjiEvent::DeclarationChanged {
                        declaration: self.declaration_view().expect("accepted declaration"),
                    });
                }
            }
            if !changed {
                break;
            }
        }
        if !events.is_empty() {
            self.reset_bid_pass_confirmations();
        }
        events
    }
}
