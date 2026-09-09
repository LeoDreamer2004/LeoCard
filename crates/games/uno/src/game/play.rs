use super::{ActionOutcome, GameError, GameState, UnoCard, UnoColor, UnoPlayerId, remove_card};

impl GameState {
    pub fn play_card(
        &mut self,
        player: UnoPlayerId,
        card: UnoCard,
        chosen_color: Option<UnoColor>,
    ) -> Result<ActionOutcome, GameError> {
        let validated = self.validate_play(player, card, chosen_color)?;

        self.uno_exposed[player.0] = false;
        self.uno_declared[player.0] = false;
        self.drawn_card = None;
        self.jump_in_open = false;
        remove_card(&mut self.players[player.0].hand, card);
        self.discard_pile.push(card);
        self.current_color = if validated.deferred_color {
            None
        } else {
            chosen_color.or(card.color())
        };
        if self.players[player.0].hand.is_empty() {
            self.pending_finisher.get_or_insert(player);
        }

        let (next_player, effect) = self.resolve_played_effect(
            player,
            card,
            chosen_color,
            validated.declared_uno,
            validated.wild_draw_was_legal,
        )?;
        self.finalize_play(player, card, next_player, validated.declared_uno);
        Ok(ActionOutcome::Played {
            player,
            card,
            next_player,
            effect,
        })
    }
}
