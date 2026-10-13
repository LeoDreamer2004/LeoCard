use super::decision::Policy;
use leocard_uno::{GameState, UnoCard, UnoColor, UnoFace, UnoFlipSide, UnoPlayerId};
use std::cmp::Reverse;

impl Policy {
    pub fn preferred_color(game: &GameState, player: UnoPlayerId) -> UnoColor {
        let colors = match game.flip_side() {
            Some(UnoFlipSide::Dark) => UnoColor::DARK,
            Some(UnoFlipSide::Light) | None => UnoColor::LIGHT,
        };
        let mut counts = [0_u8; 4];
        if let Some(state) = game.player(player) {
            for color in state.hand().iter().filter_map(|card| card.color()) {
                if let Some(index) = colors.iter().position(|candidate| *candidate == color) {
                    counts[index] = counts[index].saturating_add(1);
                }
            }
        }
        colors
            .into_iter()
            .enumerate()
            .max_by_key(|(index, _)| (counts[*index], Reverse(*index)))
            .map(|(_, color)| color)
            .unwrap_or(UnoColor::Red)
    }

    pub(super) fn automatic_chosen_color(
        game: &GameState,
        player: UnoPlayerId,
        card: UnoCard,
    ) -> Option<UnoColor> {
        matches!(
            card.face(),
            UnoFace::Wild
                | UnoFace::DarkWild
                | UnoFace::WildDrawFour
                | UnoFace::WildDrawTwo
                | UnoFace::WildDrawColor
                | UnoFace::WildPowerReverse
                | UnoFace::WildNoU
                | UnoFace::WildStackThree
                | UnoFace::WildStackNumber
                | UnoFace::WildReverseDrawFour
                | UnoFace::WildDrawSix
                | UnoFace::WildDrawTen
        )
        .then(|| Self::preferred_color(game, player))
    }
}
