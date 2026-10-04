use super::TurnClockLabel;
use crate::app::presentation::add_text;
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use leocard_protocol::{GamePhaseView, PlayerId, QiGui523Snapshot, TurnTimerView};

const CLOCK_COLOR: Color = Color::srgb(0.73, 0.69, 0.94);

pub(crate) fn turn_clock_visible(game: &QiGui523Snapshot, player: PlayerId) -> bool {
    matches!(&game.phase, GamePhaseView::Playing)
        && game.turn_timer.is_some_and(|timer| timer.player == player)
        && game
            .trick
            .as_ref()
            .is_some_and(|trick| trick.current_player == player)
        && game
            .players
            .iter()
            .find(|state| state.id == player)
            .is_some_and(|state| !state.auto_play)
}

pub(super) fn add_turn_clock(
    commands: &mut Commands,
    parent: Entity,
    timer: Option<TurnTimerView>,
    assets: &UiAssets,
) {
    let size = if timer.is_some_and(|timer| timer.base_seconds == 0) {
        16.0
    } else {
        20.0
    };
    let label = add_text(
        commands,
        parent,
        turn_timer_label(timer),
        size,
        CLOCK_COLOR,
        assets,
    );
    commands.entity(label).insert((
        TurnClockLabel,
        TextShadow {
            offset: Vec2::new(1.0, 1.5),
            color: Color::BLACK.with_alpha(0.8),
        },
    ));
}

pub(crate) fn turn_timer_label(timer: Option<TurnTimerView>) -> String {
    match timer {
        Some(timer) if timer.base_seconds > 0 => timer.base_seconds.to_string(),
        Some(timer) => format!("烧条中... {}", timer.reserve_seconds),
        None => String::new(),
    }
}
