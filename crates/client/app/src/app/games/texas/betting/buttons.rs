use super::super::{TexasTableLayout, TexasUiAction};
use crate::app::presentation::{
    ButtonKind, GameButtonImageMode, GameButtonSpec, add_textured_game_button,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_ui::UiComposer;

pub(super) fn add_texas_action_button(
    commands: &mut UiComposer,
    parent: Entity,
    label: &str,
    action: TexasUiAction,
    kind: ButtonKind,
    size: Vec2,
    assets: &UiAssets,
) -> Entity {
    let Vec2 {
        x: width,
        y: height,
    } = size;
    let (normal, hovered) = match kind {
        ButtonKind::Primary => (
            &assets.controls.game_play_button,
            &assets.controls.game_play_button_hover,
        ),
        ButtonKind::Secondary => (
            &assets.controls.game_hint_button,
            &assets.controls.game_hint_button_hover,
        ),
        ButtonKind::Warning => (
            &assets.controls.game_warning_button,
            &assets.controls.game_warning_button_hover,
        ),
        ButtonKind::Pass => (
            &assets.controls.game_pass_button,
            &assets.controls.game_pass_button_hover,
        ),
    };
    let (button, label_entity) = add_textured_game_button(
        commands,
        parent,
        assets,
        GameButtonSpec {
            label,
            action: Some(UiAction::Texas(action)),
            normal,
            hovered,
            width: width * TexasTableLayout::ACTION_SCALE,
            height: height * TexasTableLayout::ACTION_SCALE,
            font_size: 17.0 * TexasTableLayout::ACTION_SCALE,
            image_mode: GameButtonImageMode::Sliced {
                border: 30.0,
                corner_scale: 0.42 * TexasTableLayout::ACTION_SCALE,
            },
        },
    );
    commands.entity(button).insert(Node {
        width: px(width * TexasTableLayout::ACTION_SCALE),
        min_width: px(width * TexasTableLayout::ACTION_SCALE),
        height: px(height * TexasTableLayout::ACTION_SCALE),
        min_height: px(height * TexasTableLayout::ACTION_SCALE),
        flex_shrink: 0.0,
        padding: UiRect::axes(
            px(9.0 * TexasTableLayout::ACTION_SCALE),
            px(4.0 * TexasTableLayout::ACTION_SCALE),
        ),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    });
    commands.entity(label_entity).insert((
        Node {
            margin: UiRect::ZERO,
            align_self: AlignSelf::Center,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
    button
}
