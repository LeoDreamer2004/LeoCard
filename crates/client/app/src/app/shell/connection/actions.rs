//! 建房、加入房间与连接身份设置。

use super::super::{
    ChatPanelState, DomainUiAction, PressedUiAction, UiAction, UiActionHandler, UiState,
    dispatch_domain_actions, start_avatar_picker,
};
use super::can_edit_identity;
use super::form::{ConnectionField, InputField};
use super::lan::LanDiscovery;
use crate::app::runtime::{
    AppearancePreferences, AvatarImages, AvatarPicker, ClientResource, ConnectionDraft,
    HostRulePreferences, PageErrorState, save_appearance_preferences, save_connection_draft,
};
use bevy::ecs::system::SystemParam;
use bevy::log::warn;
#[cfg(test)]
use bevy::prelude::default;
use bevy::prelude::*;
use bevy::text::EditableText;
use leocard_client::{LocalPlayerConnection, LocalPlayerProfile, PlayerEconomy, TcpGameClient};
use leocard_protocol::{GameKind, MAX_PLAYER_NAME_CHARS, PlayerGender};
use leocard_ui::UiComposer;

#[derive(Clone)]
pub(crate) enum ConnectionUiAction {
    SelectCategory(super::HomeCategory),
    ToggleIdentity,
    ToggleJoin,
    CreateRoom(GameKind),
    JoinRoom,
    OpenLan,
    CloseLan,
    RefreshLan,
    JoinDiscovered(u64),
    ChooseAvatar,
    ClearAvatar,
    ToggleGender,
}

impl DomainUiAction for ConnectionUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::Connection(action) = action else {
            return None;
        };
        Some(action)
    }
}

#[derive(SystemParam)]
struct AvatarUiResources<'w> {
    avatar_images: ResMut<'w, AvatarImages>,
    avatar_picker: ResMut<'w, AvatarPicker>,
}

#[derive(SystemParam)]
pub(crate) struct ConnectionActionContext<'w, 's> {
    connection: ResMut<'w, ConnectionDraft>,
    fields: Query<'w, 's, (&'static ConnectionField, &'static EditableText)>,
    client: Option<Res<'w, ClientResource>>,
    appearance: ResMut<'w, AppearancePreferences>,
    host_rules: Res<'w, HostRulePreferences>,
    lan: Res<'w, LanDiscovery>,
    page_error: ResMut<'w, PageErrorState>,
    profile: Res<'w, LocalPlayerProfile>,
    economy: ResMut<'w, PlayerEconomy>,
    ui: ResMut<'w, UiState>,
    chat: ResMut<'w, ChatPanelState>,
    local: AvatarUiResources<'w>,
    commands: UiComposer<'w, 's>,
}

pub(crate) fn dispatch_connection_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: ConnectionActionContext,
) {
    dispatch_domain_actions::<ConnectionUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<ConnectionActionContext<'_, '_>> for ConnectionUiAction {
    fn handle(&self, context: &mut ConnectionActionContext<'_, '_>) {
        if !can_edit_identity(context.client.as_deref())
            && matches!(
                self,
                Self::ChooseAvatar | Self::ClearAvatar | Self::ToggleGender | Self::ToggleIdentity
            )
        {
            return;
        }
        context.ui.dirty = true;
        match self {
            Self::SelectCategory(category) => context.ui.home.category = *category,
            Self::ToggleIdentity => {
                context.ui.home.identity_open = !context.ui.home.identity_open;
                context.ui.home.join_open = false;
                if !context.ui.home.identity_open {
                    context.page_error.error =
                        save_connection_draft(&context.connection, &mut context.economy).err();
                }
            }
            Self::ToggleJoin => {
                context.ui.home.join_open = !context.ui.home.join_open;
                context.ui.home.identity_open = false;
                context.page_error.error =
                    save_connection_draft(&context.connection, &mut context.economy).err();
            }

            ConnectionUiAction::CreateRoom(game_kind) => create_room(*game_kind, context),
            ConnectionUiAction::JoinRoom => join_room(context),
            ConnectionUiAction::OpenLan => {
                if !context.ui.home.join_open {
                    return;
                }
                // Switching panes retires the editor before its next PostUpdate publish.
                if let Some((_, editor)) = context
                    .fields
                    .iter()
                    .find(|(field, _)| matches!(field.0, InputField::JoinAddress))
                {
                    context.connection.join_address = editor.value().to_string();
                }
                context.ui.lan.open = true;
                context.ui.lan.scroll_y = 0.0;
                context.ui.lan.generation = context.ui.lan.generation.wrapping_add(1);
            }
            ConnectionUiAction::CloseLan => context.ui.lan.open = false,
            ConnectionUiAction::RefreshLan => {
                context.ui.lan.scroll_y = 0.0;
                context.ui.lan.generation = context.ui.lan.generation.wrapping_add(1);
            }
            ConnectionUiAction::JoinDiscovered(id) => {
                if let Some(room) = context
                    .lan
                    .rooms()
                    .iter()
                    .find(|room| room.server_id == *id)
                {
                    context.connection.join_address = room.address.to_string();
                    join_room(context);
                } else {
                    context.page_error.error = Some("此房间已离线，请刷新后重试".to_owned());
                }
            }
            ConnectionUiAction::ChooseAvatar => {
                if context.local.avatar_picker.pending.is_none() {
                    match start_avatar_picker() {
                        Ok(receiver) => {
                            context.local.avatar_picker.pending = Some(receiver);
                            context.page_error.error = None;
                        }
                        Err(error) => context.page_error.error = Some(error),
                    }
                }
            }
            ConnectionUiAction::ClearAvatar => {
                context.appearance.avatar_png = None;
                context.page_error.error = save_appearance_preferences(&context.appearance).err();
            }
            ConnectionUiAction::ToggleGender => {
                context.connection.gender = match context.connection.gender {
                    PlayerGender::Male => PlayerGender::Female,
                    PlayerGender::Female => PlayerGender::Male,
                };
                context.page_error.error =
                    save_connection_draft(&context.connection, &mut context.economy).err();
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

fn create_room(game_kind: GameKind, context: &mut ConnectionActionContext<'_, '_>) {
    let result = validated_host_form(&context.connection).and_then(|(name, port)| {
        let player = LocalPlayerConnection::from_profile(
            &name,
            context.appearance.avatar_png.clone(),
            &context.profile,
            context.connection.gender,
            context.economy.frame_for_name(&name),
        );
        TcpGameClient::host_with_profile(port, context.host_rules.game_rules(game_kind), player)
            .map_err(|error| error.to_string())
    });
    finish_connection(result, context);
}

fn join_room(context: &mut ConnectionActionContext<'_, '_>) {
    let name = context.connection.player_name.trim().to_owned();
    let result = if name.is_empty() {
        Err("玩家名称不能为空".to_owned())
    } else if name.chars().count() > MAX_PLAYER_NAME_CHARS {
        Err(format!("玩家名称不能超过 {MAX_PLAYER_NAME_CHARS} 个字符"))
    } else {
        let player = LocalPlayerConnection::from_profile(
            &name,
            context.appearance.avatar_png.clone(),
            &context.profile,
            context.connection.gender,
            context.economy.frame_for_name(&name),
        );
        TcpGameClient::join_with_profile(context.connection.join_address.trim(), player)
            .map_err(|error| error.to_string())
    };
    finish_connection(result, context);
}

fn finish_connection(
    result: Result<TcpGameClient, String>,
    context: &mut ConnectionActionContext<'_, '_>,
) {
    match result {
        Ok(network) => {
            *context.chat = ChatPanelState::default();
            if let Err(error) = save_connection_draft(&context.connection, &mut context.economy) {
                warn!("{error}");
            }
            context.local.avatar_images.remote.clear();
            context.commands.insert_resource(ClientResource(network));
            context.ui.leaving_room = false;
            context.ui.lan.open = false;
            context.page_error.error = None;
        }
        Err(error) => context.page_error.error = Some(error),
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
