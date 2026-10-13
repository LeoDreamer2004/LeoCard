use super::super::{ShengjiSession, to_core_player};
use leocard_protocol::ShengjiEvent;
use leocard_shengji::{ShengjiBot, ShengjiCard, ShengjiPlayerId};

impl ShengjiSession {
    pub(in super::super) fn bot_open_cards(
        &self,
        player: ShengjiPlayerId,
        _dealt_card: ShengjiCard,
    ) -> Option<Vec<ShengjiCard>> {
        let participant = self.room.players.get(usize::from(player.0))?;
        let game = self.game.as_ref()?;
        if !participant.is_bot {
            return None;
        }
        ShengjiBot::declaration(
            game.bidding(),
            player,
            &game.players().get(usize::from(player.0))?.hand,
            *game.rules(),
            false,
        )
    }

    pub(in super::super) fn open_power_outage_bot_declaration(&mut self) -> Option<ShengjiEvent> {
        let bot_players = self
            .room
            .players
            .iter()
            .filter(|participant| participant.is_bot && !participant.left)
            .map(|participant| to_core_player(participant.id))
            .collect::<Vec<_>>();
        for player in bot_players {
            let cards = {
                let game = self.game.as_ref()?;
                if game.bidding().current().is_some() {
                    return None;
                }
                ShengjiBot::declaration(
                    game.bidding(),
                    player,
                    &game.players().get(usize::from(player.0))?.hand,
                    *game.rules(),
                    true,
                )
            };
            if let Some(cards) = cards
                && self.game.as_mut()?.declare(player, &cards).is_ok()
            {
                self.record_current_declaration();
                return Some(ShengjiEvent::DeclarationChanged {
                    declaration: self.declaration_view()?,
                });
            }
        }
        None
    }
}
