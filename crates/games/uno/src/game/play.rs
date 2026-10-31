use crate::{UnoCard, UnoColor};
use leocard_game_common::PlayerId;

use super::{UnoActionOutcome, UnoGameError, UnoGameState, remove_card};

impl UnoGameState {
    pub fn play_card(
        &mut self,
        player: PlayerId,
        card: UnoCard,
        chosen_color: Option<UnoColor>,
    ) -> Result<UnoActionOutcome, UnoGameError> {
        let validated = self.validate_play(player, card, chosen_color)?;

        self.uno_exposed[player.index()] = false;
        self.uno_declared[player.index()] = false;
        self.drawn_card = None;
        self.sudden_death_misses = 0;
        self.jump_in_open = false;
        remove_card(&mut self.players[player.index()].hand, card);
        self.discard_pile.push(card);
        self.current_color = if validated.deferred_color {
            None
        } else {
            chosen_color.or(card.color())
        };
        if self.players[player.index()].hand.is_empty() {
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
        Ok(UnoActionOutcome::Played {
            player,
            card,
            next_player,
            effect,
        })
    }
}
