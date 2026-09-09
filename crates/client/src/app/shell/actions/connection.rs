//! 建房、加入房间与连接身份设置。

use super::super::{ChatPanelState, DeveloperHandInput, UiState, start_avatar_picker};
use super::{
    AvatarUiResources, ConnectionUiAction, PressedUiAction, UiActionHandler,
    dispatch_domain_actions,
};
use crate::app::runtime::{
    AppearancePreferences, AvatarImages, ClientResource, ConnectionDraft, HostRulePreferences,
    PageErrorState, save_appearance_preferences, save_connection_draft,
};
use bevy::ecs::system::SystemParam;
use bevy::log::warn;
#[cfg(test)]
use bevy::prelude::default;
use bevy::prelude::*;
use leocard_client::{LocalPlayerConnection, LocalPlayerProfile, TcpGameClient};
use leocard_protocol::{GameKind, MAX_PLAYER_NAME_CHARS};

#[derive(SystemParam)]
pub(crate) struct ConnectionActionContext<'w, 's> {
    connection: ResMut<'w, ConnectionDraft>,
    appearance: ResMut<'w, AppearancePreferences>,
    host_rules: Res<'w, HostRulePreferences>,
    page_error: ResMut<'w, PageErrorState>,
    profile: Res<'w, LocalPlayerProfile>,
    ui: ResMut<'w, UiState>,
    chat: ResMut<'w, ChatPanelState>,
    developer_hand: ResMut<'w, DeveloperHandInput>,
    local: AvatarUiResources<'w>,
    commands: Commands<'w, 's>,
}

pub(crate) fn dispatch_connection_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: ConnectionActionContext,
) {
    dispatch_domain_actions::<ConnectionUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<ConnectionActionContext<'_, '_>> for ConnectionUiAction {
    fn handle(&self, context: &mut ConnectionActionContext<'_, '_>) {
        let connection = &mut *context.connection;
        let appearance = &mut *context.appearance;
        let host_rules = &*context.host_rules;
        let page_error = &mut *context.page_error;
        let ui = &mut *context.ui;
        let chat = &mut *context.chat;
        let developer_hand = &mut *context.developer_hand;
        let local = &mut context.local;
        match self {
            ConnectionUiAction::FocusInput(field) => {
                chat.focused = false;
                developer_hand.focused = false;
                connection.active = *field;
                page_error.error = None;
            }
            ConnectionUiAction::OpenHostGamePicker => match validated_host_form(connection) {
                Ok(_) => {
                    ui.navigation.host_game_picker_open = true;
                    ui.navigation.profile_open = false;
                    ui.navigation.player_profile = None;
                    ui.navigation.settings_open = false;
                    page_error.error = None;
                }
                Err(error) => page_error.error = Some(error),
            },
            ConnectionUiAction::CloseHostGamePicker => ui.navigation.host_game_picker_open = false,
            ConnectionUiAction::CreateRoom(game_kind) => create_room(
                *game_kind,
                connection,
                appearance,
                host_rules,
                page_error,
                &context.profile,
                ui,
                chat,
                &mut local.avatar_images,
                &mut context.commands,
            ),
            ConnectionUiAction::JoinRoom => join_room(
                connection,
                appearance,
                page_error,
                &context.profile,
                ui,
                chat,
                &mut local.avatar_images,
                &mut context.commands,
            ),
            ConnectionUiAction::ChooseAvatar => {
                if local.avatar_picker.pending.is_none() {
                    match start_avatar_picker() {
                        Ok(receiver) => {
                            local.avatar_picker.pending = Some(receiver);
                            page_error.error = None;
                        }
                        Err(error) => page_error.error = Some(error),
                    }
                }
            }
            ConnectionUiAction::ClearAvatar => {
                appearance.avatar_png = None;
                page_error.error = save_appearance_preferences(appearance).err();
            }
        }
    }
}

fn validated_host_form(form: &ConnectionDraft) -> Result<(String, u16), String> {
    let name = form.player_name.trim().to_owned();
    if name.is_empty() {
        return Err("玩家名称不能为空".to_owned());
    }
    if name.chars().count() > MAX_PLAYER_NAME_CHARS {
        return Err(format!("玩家名称不能超过 {MAX_PLAYER_NAME_CHARS} 个字符"));
    }
    let port = form
        .host_port
        .trim()
        .parse::<u16>()
        .ok()
        .filter(|port| *port > 0)
        .ok_or_else(|| "端口必须是 1 到 65535 之间的整数".to_owned())?;
    Ok((name, port))
}

fn create_room(
    game_kind: GameKind,
    connection: &mut ConnectionDraft,
    appearance: &AppearancePreferences,
    host_rules: &HostRulePreferences,
    page_error: &mut PageErrorState,
    profile: &LocalPlayerProfile,
    ui: &mut UiState,
    chat: &mut ChatPanelState,
    avatars: &mut AvatarImages,
    commands: &mut Commands,
) {
    let result = validated_host_form(connection).and_then(|(name, port)| {
        let player =
            LocalPlayerConnection::from_profile(&name, appearance.avatar_png.clone(), profile);
        TcpGameClient::host_with_profile(port, host_rules.game_rules(game_kind), player)
            .map_err(|error| error.to_string())
    });
    finish_connection(result, connection, page_error, ui, chat, avatars, commands);
}

fn join_room(
    connection: &mut ConnectionDraft,
    appearance: &AppearancePreferences,
    page_error: &mut PageErrorState,
    profile: &LocalPlayerProfile,
    ui: &mut UiState,
    chat: &mut ChatPanelState,
    avatars: &mut AvatarImages,
    commands: &mut Commands,
) {
    let name = connection.player_name.trim().to_owned();
    let result = if name.is_empty() {
        Err("玩家名称不能为空".to_owned())
    } else if name.chars().count() > MAX_PLAYER_NAME_CHARS {
        Err(format!("玩家名称不能超过 {MAX_PLAYER_NAME_CHARS} 个字符"))
    } else {
        let player =
            LocalPlayerConnection::from_profile(&name, appearance.avatar_png.clone(), profile);
        TcpGameClient::join_with_profile(&connection.join_address, player)
            .map_err(|error| error.to_string())
    };
    finish_connection(result, connection, page_error, ui, chat, avatars, commands);
}

fn finish_connection(
    result: Result<TcpGameClient, String>,
    connection: &ConnectionDraft,
    page_error: &mut PageErrorState,
    ui: &mut UiState,
    chat: &mut ChatPanelState,
    avatars: &mut AvatarImages,
    commands: &mut Commands,
) {
    match result {
        Ok(network) => {
            *chat = ChatPanelState::default();
            if let Err(error) = save_connection_draft(connection) {
                warn!("{error}");
            }
            avatars.remote.clear();
            commands.insert_resource(ClientResource(network));
            ui.navigation.host_game_picker_open = false;
            ui.leaving_room = false;
            page_error.error = None;
        }
        Err(error) => page_error.error = Some(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_form_requires_an_identity_and_valid_port() {
        let mut form = ConnectionDraft {
            player_name: "房主".to_owned(),
            host_port: "52300".to_owned(),
            ..default()
        };
        assert_eq!(validated_host_form(&form), Ok(("房主".to_owned(), 52300)));

        form.host_port = "0".to_owned();
        assert!(validated_host_form(&form).is_err());
        form.host_port = "52300".to_owned();
        form.player_name.clear();
        assert!(validated_host_form(&form).is_err());
    }
}
