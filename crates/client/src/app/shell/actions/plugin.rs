//! 按钮动作采集、领域系统注册与公共分发机制。

use super::super::{PageMotion, UiState};
use super::{ButtonActions, PressedUiAction, UiActionHandler};
use crate::app::presentation::UiPress;
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
    mut presses: MessageReader<UiPress>,
    buttons: ButtonActions,
    mut actions: MessageWriter<PressedUiAction>,
    mut ui: ResMut<UiState>,
    page_motion: Option<Res<PageMotion>>,
) {
    if page_motion.is_some_and(|motion| motion.active()) {
        presses.clear();
        return;
    }
    for press in presses.read() {
        let Ok(action) = buttons.get(press.0) else {
            continue;
        };
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
    use bevy::{picking::hover::PickingInteraction, ui_widgets::Button};

    #[derive(Resource, Default)]
    struct ActionCount(usize);

    fn count_actions(mut actions: MessageReader<PressedUiAction>, mut count: ResMut<ActionCount>) {
        count.0 += actions.read().count();
    }

    #[test]
    fn rebuilt_button_during_held_mouse_press_does_not_repeat_action() {
        let mut app = App::new();
        app.init_resource::<UiState>()
            .add_message::<UiPress>()
            .init_resource::<ActionCount>()
            .add_message::<PressedUiAction>()
            .add_systems(Update, (collect_pressed_ui_actions, count_actions).chain());

        let old = app
            .world_mut()
            .spawn((
                Button,
                PickingInteraction::Pressed,
                UiAction::Navigation(NavigationUiAction::ToggleSettings),
            ))
            .id();
        app.world_mut().write_message(UiPress(old));
        app.update();
        assert_eq!(app.world().resource::<ActionCount>().0, 1);

        app.world_mut().despawn(old);
        app.world_mut().spawn((
            Button,
            PickingInteraction::Pressed,
            UiAction::Navigation(NavigationUiAction::ToggleSettings),
        ));
        app.update();
        assert_eq!(app.world().resource::<ActionCount>().0, 1);
    }
}
