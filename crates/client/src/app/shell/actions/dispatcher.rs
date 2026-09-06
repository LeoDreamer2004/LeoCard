//! Bevy 按钮系统入口；具体动作交由所属功能域处理。

use super::*;
use crate::app::games::{mahjong, qigui523, shengji, texas_holdem, uno};
use crate::app::shell::{chat, social};
use bevy::ecs::system::SystemParam;

#[derive(SystemParam)]
pub struct ButtonActionResources<'w> {
    client: Option<ResMut<'w, ClientResource>>,
    form: ResMut<'w, ConnectionForm>,
    profile: Res<'w, LocalPlayerProfile>,
    ui: ResMut<'w, UiState>,
    shengji_presentation: ResMut<'w, ShengjiPresentationState>,
    local: LocalUiResources<'w>,
    interaction_cooldown: ResMut<'w, PlayerInteractionCooldown>,
    chat: ResMut<'w, ChatPanelState>,
    developer_hand: ResMut<'w, DeveloperHandInput>,
    updater: ResMut<'w, UpdateManager>,
}

pub fn handle_buttons(
    interactions: ButtonInteractions,
    resources: ButtonActionResources,
    mut app_exit: MessageWriter<AppExit>,
    no_response_hints: Query<Entity, With<NoLegalResponseHint>>,
    mut commands: Commands,
) {
    let ButtonActionResources {
        mut client,
        mut form,
        profile,
        mut ui,
        mut shengji_presentation,
        mut local,
        mut interaction_cooldown,
        mut chat,
        mut developer_hand,
        mut updater,
    } = resources;
    #[cfg(not(feature = "developer"))]
    let _ = &mut developer_hand;
    for (interaction, action) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let redraw = action_rebuilds_ui(action);
        let handled = mahjong::actions::handle_mahjong_button(action, &mut client)
            || texas_holdem::actions::handle_texas_holdem_button(action, &mut client, &mut ui)
            || uno::actions::handle_uno_button(action, &mut client, &mut ui)
            || shengji::actions::handle_shengji_button(
                action,
                &mut client,
                &mut ui,
                &mut shengji_presentation,
            )
            || qigui523::actions::handle_qigui523_button(
                action,
                &mut client,
                &mut ui,
                &no_response_hints,
                &mut commands,
            )
            || social::actions::handle_social_button(
                action,
                &mut client,
                &mut ui,
                &mut interaction_cooldown,
            )
            || chat::actions::handle_chat_button(
                action,
                &mut client,
                &mut form,
                &mut chat,
                &mut developer_hand,
            )
            || connection::handle_connection_button(
                action,
                &mut form,
                &profile,
                &mut ui,
                &mut chat,
                &mut developer_hand,
                &mut local,
                &mut commands,
            )
            || navigation::handle_navigation_button(
                action,
                &mut form,
                &mut ui,
                &mut local,
                &mut updater,
                &mut app_exit,
            )
            || lobby::handle_lobby_button(action, &mut client, &mut ui);
        debug_assert!(handled, "UI action has no handler");
        if handled && redraw {
            ui.dirty = true;
        }
    }
}

fn action_rebuilds_ui(action: &UiAction) -> bool {
    let redraw = !matches!(
        action,
        UiAction::ToggleCard
            | UiAction::ToggleShengjiCard
            | UiAction::ToggleInteractionMenu(_)
            | UiAction::SendInteraction { .. }
            | UiAction::ToggleChatPanel
            | UiAction::ToggleAutoPlay
            | UiAction::FocusChatInput
            | UiAction::ToggleQuickVoiceMenu
            | UiAction::SendQuickVoice(_)
            | UiAction::OpenGitHubRepository
            | UiAction::Pass
    );
    #[cfg(feature = "developer")]
    let redraw = redraw && !matches!(action, UiAction::FocusDeveloperHand);
    redraw
}
