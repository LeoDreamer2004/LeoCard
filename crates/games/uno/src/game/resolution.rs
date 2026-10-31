use crate::rating::{placements_with_eliminations, reference_point_deltas_for_placements};
use crate::{UnoCard, UnoFace};
use leocard_game_common::PlayerId;

use super::{UnoGameResult, UnoGameState, UnoPhase, UnoPlayerState};

impl UnoGameState {
    pub(super) fn finalize_play(
        &mut self,
        player: PlayerId,
        card: UnoCard,
        next_player: PlayerId,
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
        self.finish_pending_game();
    }

    pub(super) fn finish(&mut self, winner: PlayerId) -> UnoGameResult {
        self.clear_pending_draw();
        self.pending_skip = 0;
        self.pending_skip_everyone = false;
        self.pending_skip_source = None;
        self.skip_effect_targets.clear();
        self.skip_turns.fill(0);
        self.drawn_card = None;
        self.uno_exposed.fill(false);
        self.uno_declared.fill(false);
        self.jump_in_open = false;
        self.pending_swap = None;
        self.pending_finisher = None;
        self.sudden_death_misses = 0;
        let hand_scores = self
            .players
            .iter()
            .map(UnoPlayerState::hand_score)
            .collect::<Vec<_>>();
        let placements =
            placements_with_eliminations(winner, &hand_scores, &self.elimination_order)
                .expect("the recorded elimination order contains valid players");
        let result = UnoGameResult {
            winner,
            reference_deltas: reference_point_deltas_for_placements(&placements)
                .expect("validated placements"),
            placements,
            hand_scores,
        };
        self.phase = UnoPhase::Finished(result.clone());
        result
    }

    pub(super) fn finish_pending_game(&mut self) -> Option<UnoGameResult> {
        if !matches!(self.phase, UnoPhase::Playing) {
            return None;
        }
        if self.pending_swap.is_some()
            || self.pending_draw_active()
            || self.pending_skip > 0
            || !self.skip_effect_targets.is_empty()
        {
            return None;
        }
        let original = self.pending_finisher?;
        let winner = self
            .players
            .get(original.index())
            .filter(|player| !player.eliminated && player.hand.is_empty())
            .map(|player| player.id)
            .or_else(|| {
                self.players
                    .iter()
                    .find(|player| !player.eliminated && player.hand.is_empty())
                    .map(|player| player.id)
            })?;
        Some(self.finish(winner))
    }
}
