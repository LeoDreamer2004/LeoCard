//! 界面缩放。

use crate::app::presentation::{DESIGN_HEIGHT, DESIGN_WIDTH};
use crate::app::runtime::ClientResource;
use crate::app::shell::{LobbyGameMotion, PageMotion, UiState};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use leocard_client::ClientPhaseRef;

const MIN_AUTO_SCALE: f32 = if cfg!(target_os = "android") {
    0.1
} else {
    0.5
};
const MAX_AUTO_SCALE: f32 = 2.5;

pub(crate) fn ui_scale_can_change(page: Res<PageMotion>, lobby_game: Res<LobbyGameMotion>) -> bool {
    // Android pages use different design heights. Keep outgoing content in its
    // original coordinate system until composition can replace it with the next page.
    !cfg!(target_os = "android") || !page.hold_page() && !lobby_game.hold_lobby()
}

pub(crate) fn update_ui_scale(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut scale: ResMut<UiScale>,
    mut ui: ResMut<UiState>,
    client: Option<Res<ClientResource>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let design_height = if cfg!(target_os = "android")
        && !ui.shop.open
        && !ui.achievements.open
        && client
            .as_deref()
            .is_none_or(|client| !matches!(client.0.model().phase(), ClientPhaseRef::Playing(_)))
    {
        660.0
    } else {
        DESIGN_HEIGHT
    };
    let effective = (window.width() / DESIGN_WIDTH)
        .min(window.height() / design_height)
        .clamp(MIN_AUTO_SCALE, MAX_AUTO_SCALE);
    if (scale.0 - effective).abs() > 0.005 {
        scale.0 = effective;
        ui.dirty = true;
    }
}
