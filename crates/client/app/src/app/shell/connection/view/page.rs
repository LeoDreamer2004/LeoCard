//! 建房游戏选择与主连接页面。

use super::gallery::GameGallery;
use crate::app::runtime::{AppearancePreferences, AvatarImages, ConnectionDraft, UiAssets};
use crate::app::shell::HomeUiState;
use bevy::prelude::*;
use leocard_ui::{UiComposer, UiSceneKey};

pub(crate) struct ConnectionScreen<'a> {
    pub(super) connection: &'a ConnectionDraft,
    pub(super) appearance: &'a AppearancePreferences,
    pub(super) assets: &'a UiAssets,
    pub(super) avatars: &'a AvatarImages,
    pub(super) home: &'a HomeUiState,
}

impl<'a> ConnectionScreen<'a> {
    pub(crate) fn new(
        connection: &'a ConnectionDraft,
        appearance: &'a AppearancePreferences,
        assets: &'a UiAssets,
        avatars: &'a AvatarImages,
        home: &'a HomeUiState,
    ) -> Self {
        Self {
            connection,
            appearance,
            assets,
            avatars,
            home,
        }
    }

    pub(crate) fn render(self, commands: &mut UiComposer, root: Entity) {
        let canvas = commands.mount(
            root,
            "connection",
            bsn! {
                Node {
                    width: percent(100), flex_grow: 1.0,
                    min_height: px(0), justify_content: JustifyContent::Center,
                }
                Children [
                    UiSceneKey("content")
                    Node {
                        width: percent(100), max_width: px(1370),
                        min_height: px(0), flex_grow: 1.0,
                        padding: UiRect::axes(px(24), px(12)), flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart, column_gap: px(16),
                    }
                ]
            },
        );
        let content = commands.entity_for(canvas, "content");
        GameGallery::new(self.assets, self.home, &self.connection.host_port)
            .render(commands, content);
    }
}
