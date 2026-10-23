use super::{UnoBot, UnoBotObservation};
use crate::{UnoCard, UnoColor, UnoFace, UnoFlipSide};
use std::cmp::Reverse;

impl UnoBot {
    pub fn preferred_color(hand: &[UnoCard], flip_side: Option<UnoFlipSide>) -> UnoColor {
        let colors = match flip_side {
            Some(UnoFlipSide::Dark) => UnoColor::DARK,
            Some(UnoFlipSide::Light) | None => UnoColor::LIGHT,
        };
        let mut counts = [0_u8; 4];
        for color in hand.iter().filter_map(|card| card.color()) {
            if let Some(index) = colors.iter().position(|candidate| *candidate == color) {
                counts[index] = counts[index].saturating_add(1);
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
        observation: &UnoBotObservation,
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
        .then(|| Self::preferred_color(&observation.hand, observation.flip_side))
    }
}
