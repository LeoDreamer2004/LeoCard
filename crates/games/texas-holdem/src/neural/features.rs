use super::{TexasPpoBot, TexasPpoMemory};
use crate::{TexasAction, TexasGameState, TexasSuit};
use leocard_game_common::PlayerId;

#[derive(Clone, Debug, PartialEq)]
pub struct TexasPpoObservation {
    pub(super) features: Vec<f32>,
    pub(super) actions: Vec<(usize, TexasAction)>,
}

impl TexasPpoObservation {
    pub fn from_game(
        game: &TexasGameState,
        player: PlayerId,
        memory: &TexasPpoMemory,
    ) -> Option<Self> {
        if !TexasPpoBot::supports_game(game)
            || game.current_player() != Some(player)
            || game.blind_to_post().is_some()
        {
            return None;
        }
        let state = game.players().get(player.index())?;
        let mut features = vec![0.0; 84];
        for card in state.hole_cards().iter().chain(game.community()) {
            let suit = match card.suit() {
                TexasSuit::Spade => 0,
                TexasSuit::Heart => 1,
                TexasSuit::Diamond => 2,
                TexasSuit::Club => 3,
            };
            let rank = if card.rank().value() == 14 {
                0
            } else {
                card.rank().value() as usize - 1
            };
            features[suit * 13 + rank] = 1.0;
        }
        // RLCard encodes total chip contributions, without normalization.
        features[52] = state.committed_total() as f32;
        features[53] = game.players().iter().map(|p| p.committed_total()).max()? as f32;
        let mut offset = 54;
        for index in 0..6 {
            if index == player.index() {
                continue;
            }
            features[offset..offset + 6].copy_from_slice(&memory.features(index));
            offset += 6;
        }
        let call = game.amount_to_call(player).ok()?;
        let mut actions = vec![
            (0, TexasAction::Fold),
            (
                1,
                if call == 0 {
                    TexasAction::Check
                } else {
                    TexasAction::Call
                },
            ),
        ];
        let sizing = game.raise_sizing(player).ok()?;
        let maximum = state.committed_street() + state.stack();
        if game.raise_allowed(player).ok()? {
            for (index, payment) in [(2, game.pot() / 2), (3, game.pot())] {
                let target = state.committed_street() + payment;
                // Preserve the trained payment, rounding only to our legal raise steps.
                if payment < state.stack()
                    && target > game.current_bet()
                    && let Some(target) = sizing.normalize(target)
                    && target < maximum
                {
                    actions.push((index, TexasAction::RaiseTo(target)));
                }
            }
        }
        if game.all_in_allowed(player).ok()? {
            actions.push((4, TexasAction::AllIn));
        }
        Some(Self { features, actions })
    }
}
