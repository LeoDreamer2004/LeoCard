use crate::app::presentation::motion::ease_out_cubic;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_ui::{TextureButton, TransitionVisuals, UiComposer, fade_panel};

#[derive(Component)]
pub(crate) struct GameButtonEntrance(Timer);

#[track_caller]
pub(super) fn mount_game_button(
    button: TextureButton<'_, UiAction>,
    commands: &mut UiComposer,
    parent: Entity,
    font: &Handle<Font>,
) -> (Entity, Entity) {
    let (button, label) = button.mount(commands, parent, font);
    commands
        .entity(button)
        .init(GameButtonEntrance(Timer::from_seconds(
            0.14,
            TimerMode::Once,
        )));
    (button, label)
}

pub(crate) fn animate_game_button_entrances(
    time: Res<Time>,
    mut commands: UiComposer,
    mut buttons: Query<(Entity, &mut GameButtonEntrance, &mut UiTransform)>,
    children: Query<&Children>,
    mut visuals: TransitionVisuals,
) {
    for (entity, mut entrance, mut transform) in &mut buttons {
        if entrance.0.is_finished() {
            continue;
        }
        if !entrance.is_added() {
            entrance.0.tick(time.delta());
        }
        let progress = ease_out_cubic(entrance.0.fraction());
        // 按压反馈使用纵向位移和缩放，入场只移动横向位置。
        transform.translation.x = px(-24.0 * (1.0 - progress));
        fade_panel(entity, progress, &children, &mut visuals, &mut commands);
    }
}
