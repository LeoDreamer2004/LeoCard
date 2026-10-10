//! Timed click-to-continue prompt.

use super::*;
use bevy::picking::Pickable;
use bevy::ui_widgets::Button;

use crate::app::presentation::{TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;

#[expect(
    clippy::too_many_arguments,
    reason = "the prompt has explicit timing and action"
)]
pub(super) fn render_settlement_continue(
    commands: &mut Commands,
    content: Entity,
    assets: &UiAssets,
    elapsed: f32,
    start: f32,
    end: f32,
    label: &str,
    action: Option<UiAction>,
) {
    let area = spawn_node(
        commands,
        content,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.001)),
    );
    commands.entity(area).insert((
        MahjongScoreStage { start, end },
        GlobalZIndex(1300),
        Pickable::default(),
        if elapsed >= start && elapsed < end {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
    ));
    if let Some(action) = action {
        commands.entity(area).insert((Button, action));
    }
    let prompt = add_text(commands, area, label, 18.0, TEXT, assets);
    commands.entity(prompt).insert((
        Node {
            position_type: PositionType::Absolute,
            right: px(30),
            bottom: px(22),
            ..default()
        },
        TextShadow {
            offset: Vec2::new(1.0, 2.0),
            color: Color::BLACK.with_alpha(0.9),
        },
        Pickable::IGNORE,
    ));
}
