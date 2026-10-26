//! 应用皮肤专用的箭头动画，以及交互控件的点击音效。
use crate::app::runtime::UiAssets;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use leocard_ui::{UiPress, UiPressTarget};

/// The feature supplies its own click sound.
#[derive(Component, Default)]
pub(crate) struct SilentButton;

#[derive(Component)]
pub(crate) struct ButtonArrows {
    pub(crate) left: bool,
    pub(crate) owner: Entity,
    pub(crate) elapsed: f32,
}

pub(crate) fn animate_button_arrows(
    time: Res<Time>,
    interactions: Query<&PickingInteraction>,
    mut images: Query<(&mut ButtonArrows, &mut ImageNode)>,
) {
    for (mut arrow, mut image) in &mut images {
        if !interactions
            .get(arrow.owner)
            .is_ok_and(|state| *state != PickingInteraction::None)
        {
            arrow.elapsed = 0.0;
            continue;
        }
        arrow.elapsed += time.delta_secs();
        let tick = ((arrow.elapsed * 20.0) as usize) % 33;
        let frame = if tick < 16 {
            tick
        } else if tick < 19 {
            15
        } else {
            33 - tick
        } as f32;
        let left = if arrow.left { 8.0 } else { 136.0 };
        let rect = Rect::from_corners(
            Vec2::new(left, frame * 80.0 + 8.0),
            Vec2::new(left + 112.0, frame * 80.0 + 64.0),
        );
        if image.rect != Some(rect) {
            image.rect = Some(rect);
        }
    }
}

pub(crate) fn play_ui_click_sounds(
    mut presses: MessageReader<UiPress>,
    controls: Query<(), (With<UiPressTarget>, Without<SilentButton>)>,
    assets: Res<UiAssets>,
    mut commands: Commands,
) {
    if assets.audio.button_click_sounds.is_empty() {
        return;
    }
    for press in presses.read() {
        if !controls.contains(press.0) {
            continue;
        }
        let sound = assets.audio.button_click_sounds
            [fastrand::usize(..assets.audio.button_click_sounds.len())]
        .clone();
        commands.spawn((AudioPlayer::new(sound), PlaybackSettings::DESPAWN));
    }
}
