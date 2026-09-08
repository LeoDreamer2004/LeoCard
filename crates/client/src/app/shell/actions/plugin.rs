//! 按钮动作采集、领域系统注册与公共分发机制。

use super::*;
use crate::app::games::{mahjong, qigui523, shengji, texas_holdem, uno};
use crate::app::shell::{chat, social};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub struct UiActionSet;

pub struct UiActionPlugin;

impl Plugin for UiActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PressedUiAction>().add_systems(
            Update,
            (
                collect_pressed_ui_actions,
                (
                    mahjong::actions::dispatch_mahjong_actions,
                    texas_holdem::actions::dispatch_texas_holdem_actions,
                    uno::actions::dispatch_uno_actions,
                    shengji::actions::dispatch_shengji_actions,
                    qigui523::actions::dispatch_qigui523_actions,
                    social::actions::dispatch_social_actions,
                    chat::actions::dispatch_chat_actions,
                    connection::dispatch_connection_actions,
                    navigation::dispatch_navigation_actions,
                    lobby::dispatch_lobby_actions,
                ),
            )
                .chain()
                .in_set(UiActionSet),
        );
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

pub fn dispatch_domain_actions<Action, Context>(
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
