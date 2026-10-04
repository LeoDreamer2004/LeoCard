use super::{
    UnoActionContext, UnoMatchStatistics,
    state::{DrawChain, FinishCandidate},
};
use crate::{ActionOutcome, GameState, PlayedEffect, UnoCard, UnoColor, UnoFace};

impl UnoMatchStatistics {
    pub(super) fn record_plays(
        &mut self,
        before: &UnoActionContext,
        outcome: &ActionOutcome,
        played: &[(UnoCard, Option<UnoColor>)],
        jump_in: bool,
        game: &GameState,
    ) {
        let actor = before.player;
        if let ActionOutcome::ColorChosen { player, color }
        | ActionOutcome::ColorRouletteResolved { player, color, .. } = outcome
        {
            self.color_change = (before.color != Some(*color)).then_some((*player, *color));
        }
        if played.is_empty() {
            return;
        }
        let tracked = &mut self.players[actor.0];
        let color_gift = before.hands[actor.0] == 1
            && played.len() == 1
            && played[0].0.color().is_some_and(|color| {
                Some(color) == before.color
                    && self
                        .color_change
                        .is_some_and(|(source, gift)| source != actor && gift == color)
            });
        for (card, _) in played {
            if let UnoFace::Number(number) = card.face() {
                tracked.progress.numbers |= 1 << number;
            }
            if let Some(color) = card.color() {
                tracked.progress.colors |= 1 << (color as u8);
            }
            tracked.progress.wild_cards = tracked
                .progress
                .wild_cards
                .saturating_add(u32::from(card.face().is_wild()));
            tracked.progress.played_penalty |= penalty_card(card.face());
        }
        if jump_in {
            tracked.progress.jump_ins = tracked.progress.jump_ins.saturating_add(1);
            tracked.progress.jumped_reverse |= played
                .iter()
                .any(|(card, _)| card.face() == UnoFace::Reverse);
        }
        if game
            .player(actor)
            .is_some_and(|player| player.hand().is_empty())
        {
            tracked.finish = FinishCandidate {
                pair: played.len() == 2,
                forgotten: before.hands[actor.0] == 1 && before.exposed,
                color_gift,
            };
        }
        if game.rules().is_no_mercy()
            && played
                .iter()
                .any(|(card, _)| card.face() == UnoFace::Number(0))
            && matches!(
                outcome,
                ActionOutcome::Played {
                    effect: Some(PlayedEffect::HandsPassed { .. }),
                    ..
                }
            )
        {
            tracked.progress.hand_swaps = tracked.progress.hand_swaps.saturating_add(1);
        }
        self.color_change = game
            .current_color()
            .filter(|color| Some(*color) != before.color)
            .map(|color| (actor, color));
        let adds_penalty = played.iter().any(|(card, _)| penalty_card(card.face()));
        if adds_penalty
            && !matches!(
                played[0].0.face(),
                UnoFace::WildNoU | UnoFace::WildColorRoulette
            )
        {
            if before.turn.pending_kind.is_none() {
                self.chain = DrawChain {
                    origin: Some(actor),
                    ..Default::default()
                };
            }
            self.chain.links = self.chain.links.saturating_add(played.len() as u16);
            self.chain
                .cards
                .extend(played.iter().map(|(card, _)| card.public_face()));
        } else if matches!(
            outcome,
            ActionOutcome::Played {
                effect: Some(PlayedEffect::DrawReflected { .. }),
                ..
            }
        ) {
            self.chain.links = self.chain.links.saturating_add(1);
        }
    }
}

pub(super) fn penalty_card(face: UnoFace) -> bool {
    face.draw_value().is_some()
        || matches!(
            face,
            UnoFace::WildDrawFour
                | UnoFace::WildDrawColor
                | UnoFace::WildColorRoulette
                | UnoFace::WildNoU
                | UnoFace::StackOne
                | UnoFace::StackTwo
                | UnoFace::WildStackThree
                | UnoFace::WildStackNumber
        )
}
