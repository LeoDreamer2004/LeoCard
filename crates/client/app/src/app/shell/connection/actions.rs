//! 建房、加入房间与连接身份设置。

use super::super::{
    ChatPanelState, DomainUiAction, PageMotion, PressedUiAction, UiAction, UiActionHandler,
    UiState, dispatch_domain_actions, start_avatar_picker,
};
use super::can_edit_identity;
use super::form::{ConnectionField, InputField};
use super::lan::LanDiscovery;
use super::rooms::{JoinRoomDialog, JoinRoomDialogKind, RoomDirectory, RoomSelection};
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
    SelectRoom(RoomSelection),
    JoinSelected,
    DirectConnect,
    AddRoom,
    EditRoom,
    SaveRoom,
    DeleteRoom,
    ConfirmDeleteRoom,
    CancelRoomEdit,
    RefreshRooms,
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
    rooms: ResMut<'w, RoomDirectory>,
    time: Res<'w, Time>,
    page_motion: ResMut<'w, PageMotion>,
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
        if context.ui.rooms.connecting
            && matches!(
                self,
                Self::JoinRoom
                    | Self::JoinSelected
                    | Self::SelectRoom(_)
                    | Self::DirectConnect
                    | Self::AddRoom
                    | Self::EditRoom
                    | Self::DeleteRoom
                    | Self::SaveRoom
                    | Self::ConfirmDeleteRoom
                    | Self::RefreshRooms
            )
        {
            return;
        }
        context.ui.dirty = true;
        if !matches!(self, Self::SelectRoom(_)) {
            context.ui.rooms.last_click = None;
        }
        match self {
            Self::SelectCategory(category) => context.ui.home.category = *category,
            Self::ToggleIdentity => {
                context.ui.home.identity_open = !context.ui.home.identity_open;
                if !context.ui.home.identity_open {
                    context.page_error.error =
                        save_connection_draft(&context.connection, &mut context.economy).err();
                }
            }
            Self::ToggleJoin => {
                cancel_join(context);
                context.page_motion.begin();
                context.ui.shop.open = false;
                context.ui.achievements.open = false;
                let open = !context.ui.rooms.open;
                context.ui.rooms.set_open(open);
                context.ui.home.identity_open = false;
                context.page_error.error =
                    save_connection_draft(&context.connection, &mut context.economy).err();
            }

            ConnectionUiAction::CreateRoom(game_kind) => create_room(*game_kind, context),
            ConnectionUiAction::JoinRoom => join_room(context),
            Self::SelectRoom(selection) => {
                let now = context.time.elapsed_secs_f64();
                let double = context
                    .ui
                    .rooms
                    .last_click
                    .as_ref()
                    .is_some_and(|(last, time)| last == selection && now - time < 0.4);
                context.ui.rooms.selected = Some(selection.clone());
                context.ui.rooms.last_click = Some((selection.clone(), now));
                if double {
                    join_selected(context);
                }
            }
            Self::JoinSelected => join_selected(context),
            Self::DirectConnect => {
                context.ui.rooms.show_dialog(JoinRoomDialog {
                    kind: JoinRoomDialogKind::Direct,
                    name: String::new(),
                    address: context.connection.join_address.clone(),
                });
            }
            Self::AddRoom => open_room_editor(false, context),
            Self::EditRoom => open_room_editor(true, context),
            Self::SaveRoom => {
                capture_room_inputs(context);
                if let Some(editor) = context.ui.rooms.dialog.as_ref().filter(|dialog| {
                    matches!(
                        dialog.kind,
                        JoinRoomDialogKind::Add | JoinRoomDialogKind::Edit(_)
                    )
                }) {
                    let result = context.rooms.store.save(
                        editor.original_address(),
                        &editor.name,
                        &editor.address,
                    );
                    match result {
                        Ok(address) => {
                            context.ui.rooms.selected = Some(RoomSelection::Saved(address));
                            context.ui.rooms.dialog_open = false;
                            context.ui.rooms.generation =
                                context.ui.rooms.generation.wrapping_add(1);
                            context.page_error.error = None;
                        }
                        Err(error) => context.page_error.error = Some(error),
                    }
                }
            }
            Self::DeleteRoom => {
                if let Some(RoomSelection::Saved(address)) = context.ui.rooms.selected.as_ref()
                    && let Some(room) = context
                        .rooms
                        .entries()
                        .iter()
                        .find(|room| &room.address == address)
                {
                    let dialog = JoinRoomDialog {
                        kind: JoinRoomDialogKind::Delete(address.clone()),
                        name: room.name.clone(),
                        address: address.clone(),
                    };
                    context.ui.rooms.show_dialog(dialog);
                }
            }
            Self::ConfirmDeleteRoom => {
                if let Some(JoinRoomDialog {
                    kind: JoinRoomDialogKind::Delete(address),
                    ..
                }) = context.ui.rooms.dialog.as_ref()
                {
                    let result = context.rooms.store.remove(address);
                    match result {
                        Ok(()) => {
                            context.ui.rooms.selected = None;
                            context.ui.rooms.dialog_open = false;
                            context.ui.rooms.generation =
                                context.ui.rooms.generation.wrapping_add(1);
                        }
                        Err(error) => context.page_error.error = Some(error),
                    }
                }
            }
            Self::CancelRoomEdit => {
                cancel_join(context);
                context.ui.rooms.dialog_open = false;
            }
            Self::RefreshRooms => {
                context.ui.rooms.generation = context.ui.rooms.generation.wrapping_add(1)
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

fn cancel_join(context: &mut ConnectionActionContext<'_, '_>) {
    if context.ui.rooms.connecting {
        context.commands.remove_resource::<ClientResource>();
        context.ui.rooms.connecting = false;
        context.rooms.pending_join = None;
    }
}

fn capture_room_inputs(context: &mut ConnectionActionContext<'_, '_>) {
    if context.ui.rooms.dialog.is_none() {
        return;
    }
    for (field, editor) in &context.fields {
        match field.0 {
            InputField::JoinAddress => {
                if let Some(draft) = context.ui.rooms.dialog.as_mut() {
                    draft.address = editor.value().to_string();
                }
            }
            InputField::RoomName => {
                if let Some(draft) = context.ui.rooms.dialog.as_mut() {
                    draft.name = editor.value().to_string();
                }
            }
            _ => {}
        }
    }
}

fn open_room_editor(edit: bool, context: &mut ConnectionActionContext<'_, '_>) {
    let selected = context.ui.rooms.selected.as_ref();
    let saved = match selected {
        Some(RoomSelection::Saved(address)) => context
            .rooms
            .entries()
            .iter()
            .find(|room| &room.address == address),
        _ => None,
    };
    if edit && saved.is_none() {
        return;
    }
    let lan = match selected {
        Some(RoomSelection::Lan(id)) => context
            .lan
            .rooms()
            .iter()
            .find(|room| room.server_id == *id),
        _ => None,
    };
    let (name, address) = if let Some(room) = saved.filter(|_| edit) {
        (room.name.clone(), room.address.clone())
    } else if let Some(room) = lan {
        (
            format!("{}的房间", room.info.host_name),
            room.address.to_string(),
        )
    } else {
        (String::new(), String::new())
    };
    context.ui.rooms.show_dialog(JoinRoomDialog {
        kind: if edit {
            JoinRoomDialogKind::Edit(address.clone())
        } else {
            JoinRoomDialogKind::Add
        },
        name,
        address,
    });
}

fn join_selected(context: &mut ConnectionActionContext<'_, '_>) {
    let address = match context.ui.rooms.selected.as_ref() {
        Some(RoomSelection::Saved(address)) => Some(address.clone()),
        Some(RoomSelection::Lan(id)) => context
            .lan
            .rooms()
            .iter()
            .find(|room| room.server_id == *id)
            .map(|room| room.address.to_string()),
        None => None,
    };
    if let Some(address) = address {
        context.connection.join_address = address;
        join_room(context);
    } else {
        context.page_error.error = Some("请选择一个房间，或使用直接连接".to_owned());
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
    context.rooms.pending_join = None;
    context.ui.rooms.connecting = false;
    context.ui.rooms.open = false;
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
    if context.ui.rooms.connecting {
        return;
    }
    capture_room_inputs(context);
    if let Some(dialog) = context
        .ui
        .rooms
        .dialog
        .as_ref()
        .filter(|dialog| matches!(dialog.kind, JoinRoomDialogKind::Direct))
    {
        context.connection.join_address.clone_from(&dialog.address);
    }
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
    if result.is_ok() {
        context.rooms.pending_join = Some(context.connection.join_address.trim().to_owned());
        context.ui.rooms.connecting = true;
    }
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
            context.ui.home.identity_open = false;
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
