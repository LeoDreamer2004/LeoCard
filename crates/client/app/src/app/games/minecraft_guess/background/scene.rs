use super::assets::BackgroundAssets;
use crate::app::presentation::UiComposer;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::LayoutConfig;

#[derive(Component)]
pub(in super::super) struct GuessBackgroundPage;

#[derive(Component)]
pub(in super::super) struct GuessBackgroundCanvas;

pub(in super::super) fn add_guess_background(commands: &mut UiComposer, root: Entity) {
    commands.entity(root).insert(GuessBackgroundPage);
}

pub(super) fn setup_background(mut commands: UiComposer, assets: Res<BackgroundAssets>) {
    commands.spawn((
        GuessBackgroundCanvas,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        MaterialNode(assets.material.clone()),
        LayoutConfig {
            use_rounding: false,
        },
        GlobalZIndex(-100),
        Pickable::IGNORE,
        Visibility::Hidden,
    ));
}
