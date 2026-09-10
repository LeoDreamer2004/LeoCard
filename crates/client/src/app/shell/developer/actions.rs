//! 开发者工具按钮动作。

use super::DeveloperHandInput;
use crate::app::runtime::{ConnectionDraft, PageErrorState};
use crate::app::shell::{
    ChatPanelState, DomainUiAction, PressedUiAction, UiAction, UiActionHandler,
    dispatch_domain_actions,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

#[derive(Clone)]
pub(crate) enum DeveloperUiAction {
    FocusHandInput,
}

impl DomainUiAction for DeveloperUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::Developer(action) = action else {
            return None;
        };
        Some(action)
    }

    fn rebuilds_ui(&self) -> bool {
        false
    }
}

#[derive(SystemParam)]
pub(crate) struct DeveloperActionContext<'w> {
    connection: ResMut<'w, ConnectionDraft>,
    page_error: ResMut<'w, PageErrorState>,
    chat: ResMut<'w, ChatPanelState>,
    developer_hand: ResMut<'w, DeveloperHandInput>,
}

pub(crate) fn dispatch_developer_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: DeveloperActionContext,
) {
    dispatch_domain_actions::<DeveloperUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<DeveloperActionContext<'_>> for DeveloperUiAction {
    fn handle(&self, context: &mut DeveloperActionContext<'_>) {
        match self {
            Self::FocusHandInput => {
                context.chat.focused = false;
                context.developer_hand.focused = true;
                context.connection.active = crate::app::shell::InputField::PlayerName;
                context.page_error.error = None;
            }
        }
    }
}
