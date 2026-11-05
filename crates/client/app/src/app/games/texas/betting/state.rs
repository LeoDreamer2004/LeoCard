use bevy::prelude::*;
use leocard_protocol::{TexasPhaseView, TexasSnapshot};
use leocard_texas_holdem::TexasRaiseSizing;

#[derive(Component)]
pub(in super::super) struct TexasRaiseInput(pub TexasRaiseSizing);

#[derive(Component)]
pub(in super::super) struct TexasRaiseSlider(pub TexasRaiseSizing);

#[derive(Component)]
pub(in super::super) struct TexasRaiseFill {
    pub sizing: TexasRaiseSizing,
    pub target: u32,
}

pub(in super::super) fn raise_sizing(game: &TexasSnapshot) -> Option<TexasRaiseSizing> {
    if !matches!(game.phase, TexasPhaseView::Betting { .. })
        || game.blind_to_post.is_some()
        || game.current_player != Some(game.you)
        || !game.raise_allowed
    {
        return None;
    }
    let sizing = TexasRaiseSizing::new(
        game.current_bet,
        game.minimum_raise_to
            .saturating_sub(game.current_bet)
            .max(1),
        game.maximum_raise_to?,
    );
    sizing.maximum().map(|_| sizing)
}

pub(super) fn raise_from_fraction(sizing: TexasRaiseSizing, fraction: f32) -> u32 {
    let minimum = sizing.minimum();
    let maximum = sizing.maximum().unwrap_or(minimum);
    let target = minimum.saturating_add(
        (f64::from(maximum - minimum) * f64::from(fraction.clamp(0.0, 1.0))).round() as u32,
    );
    sizing.normalize(target).unwrap_or(minimum)
}

pub(super) fn raise_fraction(sizing: TexasRaiseSizing, target: u32) -> f32 {
    let minimum = sizing.minimum();
    let maximum = sizing.maximum().unwrap_or(minimum);
    if maximum == minimum {
        0.0
    } else {
        target.saturating_sub(minimum) as f32 / (maximum - minimum) as f32
    }
}
