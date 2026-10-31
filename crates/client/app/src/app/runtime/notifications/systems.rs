#[cfg(not(target_os = "android"))]
use super::gate::REMINDER_COOLDOWN;
use super::gate::{ActionContext, ActionReminder};
use crate::app::games::GameScreens;
#[cfg(not(target_os = "android"))]
use crate::app::runtime::APP_ICON_PNG;
use crate::app::runtime::ClientResource;
#[cfg(not(target_os = "android"))]
use crate::platform::notifications::{
    DesktopNotification, DesktopNotifications, NotificationTicket,
};
#[cfg(not(target_os = "android"))]
use crate::player::config_file;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use leocard_client::{ClientPhaseRef, NetworkState};
#[cfg(not(target_os = "android"))]
use leocard_game_common::PlayerPersona;
use std::time::Instant;

#[derive(Resource, Default)]
pub(super) struct TurnNotifications {
    reminder: ActionReminder,
    #[cfg(not(target_os = "android"))]
    service: Option<DesktopNotifications>,
    #[cfg(not(target_os = "android"))]
    ticket: Option<NotificationTicket>,
}

impl TurnNotifications {
    fn cancel_pending(&mut self) {
        #[cfg(not(target_os = "android"))]
        if let Some(ticket) = self.ticket.take() {
            ticket.cancel();
        }
    }

    fn report_errors(&self) {
        #[cfg(not(target_os = "android"))]
        if let Some(error) = self
            .service
            .as_ref()
            .and_then(DesktopNotifications::take_error)
        {
            warn!("桌面通知不可用：{error}");
        }
    }

    #[cfg(not(target_os = "android"))]
    fn show(&mut self, name: &str) {
        let Some(service) = &self.service else { return };
        match service.show(
            DesktopNotification {
                title: "LeoCard".to_owned(),
                body: reminder_body(name),
            },
            REMINDER_COOLDOWN,
        ) {
            Ok(ticket) => self.ticket = Some(ticket),
            Err(error) => warn!("{error}"),
        }
    }

    #[cfg(target_os = "android")]
    fn show(&mut self, _name: &str) {}
}

#[cfg(not(target_os = "android"))]
pub(super) fn reminder_body(name: &str) -> String {
    let name = match PlayerPersona::from(name) {
        PlayerPersona::DreamerLeo => "Dev",
        PlayerPersona::Cnofne => "群主",
        PlayerPersona::Yang120 => "官儿",
        PlayerPersona::SnowyWind => "马眼",
        PlayerPersona::Wpy03 => "盆儿",
        PlayerPersona::Regular => name,
    };
    format!("{name}，轮到你行动了！")
}

#[cfg(not(target_os = "android"))]
pub(super) fn setup_turn_notifications(mut state: ResMut<TurnNotifications>) {
    let Some(icon) = config_file("notifications/app-icon.png") else {
        warn!("找不到桌面通知图标的缓存目录");
        return;
    };
    match DesktopNotifications::new("com.leodreamer.leocard", "LeoCard", APP_ICON_PNG, icon) {
        Ok(service) => state.service = Some(service),
        Err(error) => warn!("无法启动桌面通知线程：{error}"),
    }
}

pub(super) fn update_turn_notifications(
    client: Option<Res<ClientResource>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    screens: GameScreens,
    mut state: ResMut<TurnNotifications>,
) {
    state.report_errors();
    let action = client.as_deref().and_then(|client| {
        if !matches!(client.0.state(), NetworkState::Connected(_)) {
            return None;
        }
        let model = client.0.model();
        let ClientPhaseRef::Playing(game) = model.phase() else {
            return None;
        };
        // 观战快照沿用被观察者的 you；真实本地身份必须另行核对。
        if model.you() != Some(game.you()) || !screens.requires_action(game) {
            return None;
        }
        let player = game.player(game.you())?;
        Some((
            ActionContext {
                room: model.room_id(),
                game: game.kind(),
                match_id: game.match_id(),
                player: game.you(),
            },
            player.name.as_str(),
        ))
    });
    let focused = windows.single().map_or(true, |window| window.focused);
    if focused || action.is_none() {
        state.cancel_pending();
    }
    if state
        .reminder
        .observe(action.map(|(context, _)| context), focused, Instant::now())
        && let Some((_, name)) = action
    {
        state.cancel_pending();
        state.show(name);
    }
}
