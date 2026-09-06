//! 建房、加入房间与连接身份设置。

use super::*;
use leocard_client::{LocalPlayerConnection, TcpGameClient};
use leocard_protocol::{GameKind, MAX_PLAYER_NAME_CHARS};

pub fn handle_connection_button(
    action: &UiAction,
    form: &mut ConnectionForm,
    profile: &LocalPlayerProfile,
    ui: &mut UiState,
    chat: &mut ChatPanelState,
    developer_hand: &mut DeveloperHandInput,
    local: &mut LocalUiResources<'_>,
    commands: &mut Commands,
) -> bool {
    match action {
        UiAction::FocusInput(field) => {
            chat.focused = false;
            developer_hand.focused = false;
            form.active = *field;
            form.error = None;
        }
        UiAction::OpenHostGamePicker => match validated_host_form(form) {
            Ok(_) => {
                ui.navigation.host_game_picker_open = true;
                ui.navigation.profile_open = false;
                ui.navigation.player_profile = None;
                ui.navigation.settings_open = false;
                form.error = None;
            }
            Err(error) => form.error = Some(error),
        },
        UiAction::CloseHostGamePicker => ui.navigation.host_game_picker_open = false,
        UiAction::CreateRoom(game_kind) => create_room(
            *game_kind,
            form,
            profile,
            ui,
            chat,
            &mut local.avatar_images,
            commands,
        ),
        UiAction::JoinRoom => {
            join_room(form, profile, ui, chat, &mut local.avatar_images, commands)
        }
        UiAction::ChooseAvatar => {
            if local.avatar_picker.pending.is_none() {
                match start_avatar_picker() {
                    Ok(receiver) => {
                        local.avatar_picker.pending = Some(receiver);
                        form.error = None;
                    }
                    Err(error) => form.error = Some(error),
                }
            }
        }
        UiAction::ClearAvatar => {
            form.avatar_png = None;
            form.error = save_preferences(form).err();
        }
        _ => return false,
    }
    true
}

fn validated_host_form(form: &ConnectionForm) -> Result<(String, u16), String> {
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
    form: &mut ConnectionForm,
    profile: &LocalPlayerProfile,
    ui: &mut UiState,
    chat: &mut ChatPanelState,
    avatars: &mut AvatarImages,
    commands: &mut Commands,
) {
    let result = validated_host_form(form).and_then(|(name, port)| {
        let player = LocalPlayerConnection::from_profile(&name, form.avatar_png.clone(), profile);
        match game_kind {
            GameKind::QiGui523 => TcpGameClient::host_with_profile(
                port,
                normalize_host_rules(form.host_rules),
                player,
            ),
            GameKind::TexasHoldem => TcpGameClient::host_texas_holdem_with_profile(
                port,
                normalize_texas_holdem_rules(form.texas_holdem_rules),
                player,
            ),
            GameKind::Shengji => TcpGameClient::host_shengji_with_profile(
                port,
                normalize_shengji_rules(form.shengji_rules),
                player,
            ),
            GameKind::Uno => TcpGameClient::host_uno_with_profile(
                port,
                normalize_uno_rules(form.uno_rules),
                player,
            ),
            GameKind::Mahjong => TcpGameClient::host_mahjong_with_profile(
                port,
                normalize_mahjong_rules(form.mahjong_rules),
                player,
            ),
        }
        .map_err(|error| error.to_string())
    });
    finish_connection(result, form, ui, chat, avatars, commands);
}

fn join_room(
    form: &mut ConnectionForm,
    profile: &LocalPlayerProfile,
    ui: &mut UiState,
    chat: &mut ChatPanelState,
    avatars: &mut AvatarImages,
    commands: &mut Commands,
) {
    let name = form.player_name.trim().to_owned();
    let result = if name.is_empty() {
        Err("玩家名称不能为空".to_owned())
    } else if name.chars().count() > MAX_PLAYER_NAME_CHARS {
        Err(format!("玩家名称不能超过 {MAX_PLAYER_NAME_CHARS} 个字符"))
    } else {
        let player = LocalPlayerConnection::from_profile(&name, form.avatar_png.clone(), profile);
        TcpGameClient::join_with_profile(&form.join_address, player)
            .map_err(|error| error.to_string())
    };
    finish_connection(result, form, ui, chat, avatars, commands);
}

fn finish_connection(
    result: Result<TcpGameClient, String>,
    form: &mut ConnectionForm,
    ui: &mut UiState,
    chat: &mut ChatPanelState,
    avatars: &mut AvatarImages,
    commands: &mut Commands,
) {
    match result {
        Ok(network) => {
            *chat = ChatPanelState::default();
            if let Err(error) = save_preferences(form) {
                warn!("{error}");
            }
            avatars.remote.clear();
            commands.insert_resource(ClientResource(network));
            ui.navigation.host_game_picker_open = false;
            ui.leaving_room = false;
            form.error = None;
        }
        Err(error) => form.error = Some(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_form_requires_an_identity_and_valid_port() {
        let mut form = ConnectionForm {
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
