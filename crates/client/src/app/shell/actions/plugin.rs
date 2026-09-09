//! 按钮动作采集、领域系统注册与公共分发机制。

use super::super::UiState;
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
            .add_systems(Update, collect_pressed_ui_actions.in_set(UiActionSet));
    }
}

fn collect_pressed_ui_actions(
    interactions: ButtonInteractions,
    mut actions: MessageWriter<PressedUiAction>,
    mut ui: ResMut<UiState>,
) {
    for (interaction, action) in &interactions {
        if *interaction != Interaction::Pressed {
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
