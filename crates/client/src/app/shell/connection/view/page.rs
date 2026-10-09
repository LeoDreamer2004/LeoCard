//! 建房游戏选择与主连接页面。

use super::gallery::GameGallery;
use crate::app::presentation::spawn_node;
use crate::app::runtime::{AppearancePreferences, AvatarImages, ConnectionDraft, UiAssets};
use bevy::prelude::*;

pub(crate) struct ConnectionScreen<'a> {
    pub(super) connection: &'a ConnectionDraft,
    pub(super) appearance: &'a AppearancePreferences,
    pub(super) assets: &'a UiAssets,
    pub(super) avatars: &'a AvatarImages,
}

impl<'a> ConnectionScreen<'a> {
    pub(crate) fn new(
        connection: &'a ConnectionDraft,
        appearance: &'a AppearancePreferences,
        assets: &'a UiAssets,
        avatars: &'a AvatarImages,
    ) -> Self {
        Self {
            connection,
            appearance,
            assets,
            avatars,
        }
    }

    pub(crate) fn render(self, commands: &mut Commands, root: Entity) {
        let canvas = spawn_node(
            commands,
            root,
            Node {
                width: percent(100),
                flex_grow: 1.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );

        let content = spawn_node(
            commands,
            canvas,
            Node {
                width: percent(100),
                max_width: px(1370),
                padding: UiRect::all(px(20)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                column_gap: px(16),
                ..default()
            },
            None,
        );
        GameGallery::new(self.assets).render(commands, content);
        self.render_sidebar(commands, content);
    }
}
