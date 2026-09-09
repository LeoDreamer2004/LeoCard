use super::{GameState, UnoCard, UnoFace, UnoPlayerId};

impl GameState {
    pub(super) fn finalize_play(
        &mut self,
        player: UnoPlayerId,
        card: UnoCard,
        next_player: UnoPlayerId,
        declared_uno: bool,
    ) {
        let deferred_hand_effect = self.rules.is_no_mercy()
            && (card.face() == UnoFace::Number(0) && self.rules.no_mercy.zero_pass
                || card.face() == UnoFace::Number(7) && self.rules.no_mercy.seven_swap);
        let extension_hand_effect = matches!(
            card.face(),
            UnoFace::SwapOne
                | UnoFace::RefreshHand
                | UnoFace::WildForceTrade
                | UnoFace::WildPassHands
        );
        if !extension_hand_effect && !deferred_hand_effect {
            self.update_uno_after_play(player, declared_uno);
        }

        self.current_player = next_player;
        let defers_jump_in = extension_hand_effect
            || card.face() == UnoFace::WildColorRoulette
            || deferred_hand_effect;
        if !defers_jump_in {
            self.jump_in_open = self.jump_in_enabled() && card.color().is_some();
        }
        if self.pending_swap.is_none()
            && !self.pending_draw_active()
            && self.pending_skip == 0
            && self.skip_turns[self.current_player.0] == 0
        {
            self.finish_pending_game();
        }
    }
}
