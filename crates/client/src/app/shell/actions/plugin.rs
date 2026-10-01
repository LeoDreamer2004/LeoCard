//! 按钮动作采集、领域系统注册与公共分发机制。

use super::super::{PageMotion, UiState};
use super::{ButtonInteractions, PressedUiAction, UiActionHandler};
use crate::app::runtime::ClientResource;
use bevy::prelude::*;
use leocard_protocol::{ClientCommand, GameCommand};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub(crate) struct UiActionSet;

pub(crate) struct UiActionPlugin;

impl Plugin for UiActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PressedUiAction>()
            .add_systems(Update, collect_pressed_ui_actions.before(UiActionSet));
    }
}

fn collect_pressed_ui_actions(
    interactions: ButtonInteractions,
    mouse: Res<ButtonInput<MouseButton>>,
    mut actions: MessageWriter<PressedUiAction>,
    mut ui: ResMut<UiState>,
    page_motion: Option<Res<PageMotion>>,
) {
    if page_motion.is_some_and(|motion| motion.active()) {
        return;
    }
    for (interaction, action) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        // A rebuilt button may become Pressed while the same mouse press is
        // still held. Only the physical press edge may dispatch that click.
        if mouse.pressed(MouseButton::Left) && !mouse.just_pressed(MouseButton::Left) {
            continue;
        }
        if action.rebuilds_ui() {
            ui.dirty = true;
        }
        actions.write(PressedUiAction(action.clone()));
    }
}

pub(crate) fn dispatch_domain_actions<Action, Context>(
    actions: &mut MessageReader<PressedUiAction>,
    context: &mut Context,
) where
    Action: UiActionHandler<Context>,
{
    for action in actions.read() {
        if let Some(action) = Action::extract(&action.0) {
            action.handle(context);
        }
    }
}

pub(crate) fn game_command(command: impl Into<GameCommand>) -> ClientCommand {
    ClientCommand::Game(command.into())
}

pub(crate) fn send_game_command(
    client: &mut Option<ResMut<ClientResource>>,
    command: impl Into<GameCommand>,
) {
    if let Some(client) = client.as_deref_mut() {
        client.0.send(game_command(command));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::shell::NavigationUiAction;
    use crate::app::shell::UiAction;

    #[derive(Resource, Default)]
    struct ActionCount(usize);

    fn count_actions(mut actions: MessageReader<PressedUiAction>, mut count: ResMut<ActionCount>) {
        count.0 += actions.read().count();
    }

    #[test]
    fn rebuilt_button_during_held_mouse_press_does_not_repeat_action() {
        let mut app = App::new();
        app.init_resource::<UiState>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<ActionCount>()
            .add_message::<PressedUiAction>()
            .add_systems(Update, (collect_pressed_ui_actions, count_actions).chain());

        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        let old = app
            .world_mut()
            .spawn((
                Button,
                Interaction::Pressed,
                UiAction::Navigation(NavigationUiAction::ToggleSettings),
            ))
            .id();
        app.update();
        assert_eq!(app.world().resource::<ActionCount>().0, 1);

        app.world_mut().despawn(old);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.world_mut().spawn((
            Button,
            Interaction::Pressed,
            UiAction::Navigation(NavigationUiAction::ToggleSettings),
        ));
        app.update();
        assert_eq!(app.world().resource::<ActionCount>().0, 1);
    }
}
