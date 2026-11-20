use super::JoinRoomList;
use super::store::{SavedRoom, SavedRoomStore};
use crate::app::runtime::{ClientResource, image_handle_from_png};
use crate::app::shell::connection::LanDiscovery;
use crate::app::shell::{UiState, can_edit_identity};
use crate::network::RoomInfoQueries;
use bevy::prelude::*;
use leocard_client::ClientPhaseRef;
use leocard_protocol::RoomInfo;
use std::collections::HashMap;

pub(in super::super) enum RoomStatus {
    Checking,
    Online { info: RoomInfo, latency_ms: u32 },
    Offline,
}

#[derive(Resource, Default)]
pub(crate) struct RoomDirectory {
    pub store: SavedRoomStore,
    pub pending_join: Option<String>,
    statuses: HashMap<String, RoomStatus>,
    avatars: HashMap<String, Handle<Image>>,
    queries: Vec<RoomInfoQueries>,
    generation: Option<u64>,
    cache_loaded: bool,
    last_lan_revision: Option<(Option<u64>, u64)>,
    last_connected: Option<(String, RoomInfo)>,
}

impl RoomDirectory {
    pub fn entries(&self) -> &[SavedRoom] {
        &self.store.entries
    }

    pub(in super::super) fn status(&self, address: &str) -> Option<&RoomStatus> {
        self.statuses.get(address)
    }

    pub fn avatar(&self, room: &SavedRoom) -> Option<&Handle<Image>> {
        self.avatars.get(room.host.as_ref()?.image.as_ref()?)
    }

    fn remember(&mut self, address: &str, info: &RoomInfo, images: &mut Assets<Image>) -> bool {
        let mut changed = match self.store.remember(address, info) {
            Ok(changed) => changed,
            Err(error) => {
                warn!("无法保存房间信息：{error}");
                false
            }
        };
        if let Some(key) = self
            .entries()
            .iter()
            .find(|room| room.address == address)
            .and_then(|room| room.host.as_ref()?.image.clone())
            && !self.avatars.contains_key(&key)
            && let Some(png) = self.store.avatar(&key)
            && let Some(image) = image_handle_from_png(&png, images)
        {
            self.avatars.insert(key, image);
            changed = true;
        }
        changed
    }
}

pub(in super::super) fn sync_room_directory(
    mut directory: ResMut<RoomDirectory>,
    lan: Res<LanDiscovery>,
    client: Option<Res<ClientResource>>,
    mut ui: ResMut<UiState>,
    mut images: ResMut<Assets<Image>>,
    lists: Query<&ScrollPosition, With<JoinRoomList>>,
) {
    if !directory.cache_loaded {
        let ids: Vec<_> = directory
            .entries()
            .iter()
            .filter_map(|room| room.host.as_ref()?.image.clone())
            .collect();
        for id in ids {
            if let Some(png) = directory.store.avatar(&id)
                && let Some(image) = image_handle_from_png(&png, &mut images)
            {
                directory.avatars.insert(id, image);
            }
        }
        directory.cache_loaded = true;
    }
    if let Some(address) = directory.pending_join.clone()
        && let Some(client) = client.as_deref()
        && let Some(info) = connected_info(client)
        && directory
            .last_connected
            .as_ref()
            .is_none_or(|(last_address, last_info)| last_address != &address || last_info != &info)
    {
        ui.dirty |= directory.remember(&address, &info, &mut images);
        directory.last_connected = Some((address, info));
    }
    let open = ui.rooms.open && can_edit_identity(client.as_deref());
    if !open {
        directory.queries.clear();
        directory.generation = None;
        return;
    }
    if let Some(scroll) = lists.iter().next() {
        ui.rooms.scroll_y = scroll.y;
    }
    if directory.generation != Some(ui.rooms.generation) {
        let addresses: Vec<_> = directory
            .entries()
            .iter()
            .map(|room| room.address.clone())
            .collect();
        directory.statuses = addresses
            .iter()
            .map(|address| (address.clone(), RoomStatus::Checking))
            .collect();
        directory.queries.clear();
        match RoomInfoQueries::start(addresses) {
            Ok(queries) => directory.queries.push(queries),
            Err(error) => {
                warn!("无法查询房间：{error}");
                for status in directory.statuses.values_mut() {
                    *status = RoomStatus::Offline;
                }
            }
        }
        directory.generation = Some(ui.rooms.generation);
        ui.dirty = true;
    }
    let fresh: Vec<_> = lan
        .rooms()
        .iter()
        .map(|room| room.address.to_string())
        .filter(|address| !directory.statuses.contains_key(address))
        .collect();
    if !fresh.is_empty() {
        for address in &fresh {
            directory
                .statuses
                .insert(address.clone(), RoomStatus::Checking);
        }
        match RoomInfoQueries::start(fresh.clone()) {
            Ok(queries) => directory.queries.push(queries),
            Err(_) => {
                for address in fresh {
                    directory.statuses.insert(address, RoomStatus::Offline);
                }
            }
        }
    }
    let results: Vec<_> = directory
        .queries
        .iter()
        .flat_map(RoomInfoQueries::poll)
        .collect();
    for result in results {
        if !directory.statuses.contains_key(&result.address) {
            continue;
        }
        let status = if let Some((mut info, latency_ms)) = result.info {
            if directory
                .entries()
                .iter()
                .any(|room| room.address == result.address)
            {
                directory.remember(&result.address, &info, &mut images);
            }
            info.avatar_png = None;
            RoomStatus::Online { info, latency_ms }
        } else {
            RoomStatus::Offline
        };
        directory.statuses.insert(result.address, status);
        ui.dirty = true;
    }
    if directory.last_lan_revision != Some(lan.revision()) {
        for room in lan.rooms() {
            let address = room.address.to_string();
            if directory
                .entries()
                .iter()
                .any(|saved| saved.address == address)
            {
                ui.dirty |= directory.remember(&address, &room.info, &mut images);
            }
            if let Some(RoomStatus::Online { info, .. }) = directory.statuses.get_mut(&address) {
                let mut latest = room.info.clone();
                latest.avatar_png = None;
                if *info != latest {
                    *info = latest;
                    ui.dirty = true;
                }
            }
        }
        directory.last_lan_revision = Some(lan.revision());
    }
}

fn connected_info(client: &ClientResource) -> Option<RoomInfo> {
    let model = client.0.model();
    if !matches!(
        model.phase(),
        ClientPhaseRef::Lobby(_) | ClientPhaseRef::Playing(_)
    ) {
        return None;
    }
    let (host, game, playing) = if let Some(lobby) = model.lobby() {
        (
            lobby
                .players
                .iter()
                .find(|p| Some(p.player.id) == lobby.host)
                .map(|p| &p.player)?,
            lobby.game,
            false,
        )
    } else {
        let game = model.game_snapshot()?;
        (game.player(game.host())?, game.kind(), true)
    };
    Some(RoomInfo {
        host_name: host.name.clone(),
        host_avatar: host.avatar,
        avatar_png: host.avatar.and_then(|id| model.avatars().get(&id).cloned()),
        game,
        playing,
    })
}
