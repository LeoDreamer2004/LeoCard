use super::{UnoActionContext, UnoMatchStatistics, state::DrawChain};
use crate::{UnoActionOutcome, UnoFace, UnoGameState, UnoPlayedEffect};
use leocard_game_common::PlayerId;

impl UnoMatchStatistics {
    pub(super) fn record_draws(
        &mut self,
        before: &UnoActionContext,
        outcome: &UnoActionOutcome,
        game: &UnoGameState,
    ) {
        match outcome {
            UnoActionOutcome::DrewCards { player, cards, .. } => {
                self.received(*player, cards.len(), false, before);
                self.players[player.index()].progress.lucky_elimination |=
                    before.hands[player.index()] <= 5
                        && game.player(*player).is_some_and(|state| state.eliminated());
            }
            UnoActionOutcome::PenaltyDrawn { player, cards, .. } => {
                self.received(*player, cards.len(), true, before);
                self.received_chain(*player, cards.len(), game);
            }
            UnoActionOutcome::ChallengeResolved {
                penalized, cards, ..
            } => {
                self.received(*penalized, cards.len(), true, before);
                self.received_chain(*penalized, cards.len(), game);
            }
            UnoActionOutcome::UnoReported { target, cards, .. } => {
                self.received(*target, cards.len(), true, before)
            }
            UnoActionOutcome::SkipResolved {
                player,
                cards,
                remaining,
                ..
            } => {
                self.received(*player, cards.len(), true, before);
                self.players[player.index()].progress.max_skip_batch = self.players[player.index()]
                    .progress
                    .max_skip_batch
                    .max(remaining.saturating_add(1));
                self.players[player.index()].progress.max_skipped_turns = self.players
                    [player.index()]
                .progress
                .max_skipped_turns
                .max(remaining.saturating_add(1));
            }
            UnoActionOutcome::ColorRouletteResolved { player, cards, .. } => {
                self.received(*player, cards.len(), true, before)
            }
            UnoActionOutcome::Played {
                effect: Some(UnoPlayedEffect::DrawReflected { player, cards }),
                ..
            } => {
                self.received(*player, cards.len(), true, before);
                self.received_chain(*player, cards.len(), game);
            }
            UnoActionOutcome::Played {
                player,
                effect: Some(UnoPlayedEffect::HandRefreshed { count }),
                ..
            } => self.players[player.index()].progress.has_drawn |= *count > 0,
            _ => {}
        }
    }

    fn received(
        &mut self,
        player: PlayerId,
        count: usize,
        penalty: bool,
        before: &UnoActionContext,
    ) {
        let stats = &mut self.players[player.index()].progress;
        stats.has_drawn |= count > 0;
        stats.has_penalty |= penalty && count > 0;
        if penalty {
            stats.max_penalty_cards = stats
                .max_penalty_cards
                .max(count.min(usize::from(u16::MAX)) as u16);
        }
        stats.peak_hand = stats.peak_hand.max(
            before.hands[player.index()]
                .saturating_add(count)
                .min(usize::from(u16::MAX)) as u16,
        );
    }

    fn received_chain(&mut self, player: PlayerId, count: usize, game: &UnoGameState) {
        let stats = &mut self.players[player.index()].progress;
        stats.draw_chain_returned |=
            count > 0 && self.chain.origin == Some(player) && self.chain.links >= 2;
        stats.full_draw_chain |= game.rules().is_classic()
            && count >= 32
            && self
                .chain
                .cards
                .iter()
                .filter(|card| card.face() == UnoFace::DrawTwo)
                .count()
                == 8
            && self
                .chain
                .cards
                .iter()
                .filter(|card| card.face() == UnoFace::WildDrawFour)
                .count()
                == 4;
        self.chain = DrawChain::default();
    }
}
