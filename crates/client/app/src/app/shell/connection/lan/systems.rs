use super::{LanDiscovery, LanRoomList};
use crate::app::runtime::{ClientResource, image_handle_from_png};
use crate::app::shell::UiState;
use bevy::prelude::*;
use image::{ImageFormat, ImageReader};
use leocard_client::{ClientPhaseRef, LanRoomBrowser};
use leocard_protocol::AVATAR_DIMENSION;
use std::io::Cursor;

pub(crate) fn sync_lan_discovery(
    mut discovery: ResMut<LanDiscovery>,
    mut ui: ResMut<UiState>,
    client: Option<Res<ClientResource>>,
    mut images: ResMut<Assets<Image>>,
    scrolls: Query<&ScrollPosition, With<LanRoomList>>,
) {
    if client.is_some_and(|client| {
        !matches!(
            client.0.model().phase(),
            ClientPhaseRef::Idle | ClientPhaseRef::Closed
        )
    }) {
        ui.lan.open = false;
    }
    if !ui.lan.open {
        if discovery.generation.is_some() {
            *discovery = LanDiscovery::default();
        }
        return;
    }
    if discovery.generation == Some(ui.lan.generation)
        && let Some(scroll) = scrolls.iter().next()
    {
        ui.lan.scroll_y = scroll.y;
    }
    if discovery.generation != Some(ui.lan.generation) {
        *discovery = LanDiscovery::default();
        discovery.generation = Some(ui.lan.generation);
        match LanRoomBrowser::start() {
            Ok(browser) => discovery.browser = Some(browser),
            Err(error) => discovery.error = Some(format!("无法搜索局域网房间：{error}")),
        }
        ui.dirty = true;
    }
    if !discovery.browser.as_mut().is_some_and(LanRoomBrowser::poll) {
        return;
    }
    let LanDiscovery {
        browser, avatars, ..
    } = &mut *discovery;
    let rooms = browser.as_ref().expect("polled browser").rooms();
    avatars.retain(|id, _| rooms.iter().any(|room| room.server_id == *id));
    for room in rooms {
        let Some(png) = room.info.avatar_png.as_deref() else {
            avatars.remove(&room.server_id);
            continue;
        };
        if avatars
            .get(&room.server_id)
            .is_some_and(|(id, _)| *id == room.info.host_avatar)
        {
            continue;
        }
        // Check dimensions before decoding untrusted PNG data into a GPU image.
        let valid = ImageReader::with_format(Cursor::new(png), ImageFormat::Png)
            .into_dimensions()
            .is_ok_and(|size| size == (AVATAR_DIMENSION, AVATAR_DIMENSION));
        let image = valid
            .then(|| image_handle_from_png(png, &mut images))
            .flatten();
        avatars.insert(room.server_id, (room.info.host_avatar, image));
    }
    ui.dirty = true;
}
