//! 顶栏及其连接状态、设置和个人资料入口。

use super::{LobbyUiAction, NavigationUiAction, UiAction, add_cozy_button};
use crate::app::presentation::{BORDER, MUTED, add_text, avatar_color, spawn_node};
use crate::app::runtime::{AvatarImages, ClientResource, ConnectionDraft, UiAssets};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_client::ClientPhaseRef;

pub(super) struct Header<'a> {
    client: Option<&'a ClientResource>,
    form: &'a ConnectionDraft,
    assets: &'a UiAssets,
    avatars: &'a AvatarImages,
}

impl<'a> Header<'a> {
    pub(super) fn new(
        client: Option<&'a ClientResource>,
        form: &'a ConnectionDraft,
        assets: &'a UiAssets,
        avatars: &'a AvatarImages,
    ) -> Self {
        Self {
            client,
            form,
            assets,
            avatars,
        }
    }

    pub(super) fn render(self, commands: &mut Commands, root: Entity) {
        let header = spawn_node(
            commands,
            root,
            Node {
                width: percent(100),
                height: px(64),
                border: UiRect::bottom(px(2)),
                padding: UiRect::axes(px(24), px(10)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            Some(Color::srgba(0.095, 0.105, 0.12, 0.97)),
        );
        commands
            .entity(header)
            .insert(BorderColor::all(Color::srgba(0.62, 0.59, 0.76, 0.45)));
        let left = spawn_node(
            commands,
            header,
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(14),
                ..default()
            },
            None,
        );
        add_text(
            commands,
            left,
            "LeoCard",
            25.0,
            Color::srgb(0.84, 0.81, 1.0),
            self.assets,
        );
        if let Some(host_port) = self.client.and_then(|client| client.0.model().host_port()) {
            let divider = spawn_node(
                commands,
                left,
                Node {
                    width: px(1),
                    height: px(24),
                    ..default()
                },
                Some(BORDER),
            );
            commands.entity(divider).insert(FocusPolicy::Pass);
            add_text(
                commands,
                left,
                format!("端口 {host_port}"),
                15.0,
                MUTED,
                self.assets,
            );
        }
        let right = spawn_node(
            commands,
            header,
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(14),
                ..default()
            },
            None,
        );
        add_cozy_button(
            commands,
            right,
            "游戏设置",
            UiAction::Navigation(NavigationUiAction::ToggleSettings),
            self.assets,
            px(116),
            40.0,
        );
        if self
            .client
            .is_some_and(|client| matches!(client.0.model().phase(), ClientPhaseRef::Playing(_)))
        {
            self.add_exit_button(commands, right);
        }
        self.add_profile_button(commands, right);
    }

    fn add_exit_button(&self, commands: &mut Commands, parent: Entity) {
        add_cozy_button(
            commands,
            parent,
            "退出游戏",
            UiAction::Lobby(LobbyUiAction::LeaveRoom),
            self.assets,
            px(116),
            40.0,
        );
    }

    fn add_profile_button(&self, commands: &mut Commands, parent: Entity) {
        let mut entity = commands.spawn((
            Button,
            UiAction::Navigation(NavigationUiAction::ToggleProfile),
            Node {
                width: px(44),
                height: px(44),
                min_width: px(44),
                border: UiRect::all(px(2)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(percent(50)),
                overflow: Overflow::clip(),
                ..default()
            },
            UiTransform::IDENTITY,
        ));
        entity.insert(BorderColor::all(Color::srgb(0.68, 0.65, 0.85)));
        if let Some(image) = self.avatars.local.as_ref() {
            entity.insert(ImageNode::new(image.clone()));
        } else {
            entity.insert(BackgroundColor(avatar_color(&self.form.player_name)));
        }
        let button = entity.id();
        commands.entity(parent).add_child(button);
        if self.avatars.local.is_none() {
            add_text(
                commands,
                button,
                self.form
                    .player_name
                    .chars()
                    .next()
                    .unwrap_or('玩')
                    .to_string(),
                17.5,
                Color::WHITE,
                self.assets,
            );
        }
    }
}
