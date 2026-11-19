use bevy::prelude::*;
use leocard_client::{LanRoom, LanRoomBrowser};
use leocard_protocol::AvatarId;
use std::collections::HashMap;

#[derive(Default, Resource)]
pub(crate) struct LanDiscovery {
    pub(super) browser: Option<LanRoomBrowser>,
    pub(super) generation: Option<u64>,
    pub(super) revision: u64,
    pub(super) error: Option<String>,
    pub(super) avatars: HashMap<u64, (Option<AvatarId>, Option<Handle<Image>>)>,
}

impl LanDiscovery {
    pub(in super::super) fn revision(&self) -> (Option<u64>, u64) {
        (self.generation, self.revision)
    }

    pub(crate) fn rooms(&self) -> &[LanRoom] {
        self.browser.as_ref().map_or(&[], LanRoomBrowser::rooms)
    }

    pub(in super::super) fn error(&self) -> Option<&str> {
        self.error
            .as_deref()
            .or_else(|| self.browser.as_ref().and_then(LanRoomBrowser::error))
    }

    pub(in super::super) fn avatar(&self, id: u64) -> Option<&Handle<Image>> {
        self.avatars.get(&id).and_then(|(_, image)| image.as_ref())
    }
}
