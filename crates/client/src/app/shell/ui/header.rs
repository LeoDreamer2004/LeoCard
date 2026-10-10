//! 顶栏及其连接状态、设置和个人资料入口。

use super::super::{
    AchievementEntry, CozyButtonVariant, LobbyGameHeader, LobbyUiAction, NavigationUiAction,
    PageTransitionElement, ProfileEntry, ShopEntry, UiAction, add_cozy_icon_button,
};
use crate::app::presentation::{BORDER, MUTED, TEXT, add_text, spawn_node};
use crate::app::runtime::{AvatarImages, ClientResource, ConnectionDraft, UiAssets};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_client::ClientPhaseRef;

pub(in crate::app::shell) struct Header<'a> {
    client: Option<&'a ClientResource>,
    form: &'a ConnectionDraft,
    assets: &'a UiAssets,
    avatars: &'a AvatarImages,
}

impl<'a> Header<'a> {
    pub(in crate::app::shell) fn new(
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

    pub(in crate::app::shell) fn render(self, commands: &mut Commands, root: Entity) {
        if self
            .client
            .is_some_and(|client| matches!(client.0.model().phase(), ClientPhaseRef::Playing(_)))
        {
            self.render_game_header(commands, root);
            return;
        }
        let header = spawn_node(
            commands,
            root,
            Node {
                width: percent(100),
                height: px(72),
                border: UiRect::bottom(px(2)),
                padding: UiRect::axes(px(24), px(5)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            Some(Color::srgba(0.095, 0.105, 0.12, 0.55)),
        );
        commands.entity(header).insert((
            BorderColor::all(Color::srgba(0.62, 0.59, 0.76, 0.45)),
            PageTransitionElement::header(),
            UiTransform::IDENTITY,
        ));
        if self
            .client
            .is_some_and(|client| matches!(client.0.model().phase(), ClientPhaseRef::Lobby(_)))
        {
            commands
                .entity(header)
                .insert((LobbyGameHeader, UiTransform::IDENTITY));
        }
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
            commands.entity(divider).insert(Pickable::IGNORE);
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
        if self.client.is_none_or(|client| {
            matches!(
                client.0.model().phase(),
                ClientPhaseRef::Idle | ClientPhaseRef::Closed
            )
        }) {
            ShopEntry {
                assets: self.assets,
            }
            .render(commands, right);
            AchievementEntry {
                assets: self.assets,
            }
            .render(commands, right);
        }
        self.add_labeled_icon_button(
            commands,
            right,
            UiAction::Navigation(NavigationUiAction::ToggleSettings),
            self.assets.home.settings_icon.clone(),
            CozyButtonVariant::Neutral,
            "设置",
        );
        ProfileEntry {
            form: self.form,
            assets: self.assets,
            avatars: self.avatars,
        }
        .render(commands, right, 44.0, 17.5);
    }

    fn render_game_header(&self, commands: &mut Commands, root: Entity) {
        let header = spawn_node(
            commands,
            root,
            Node {
                position_type: PositionType::Absolute,
                right: px(0),
                top: px(0),
                height: px(66),
                padding: UiRect::axes(px(12), px(4)),
                align_items: AlignItems::Center,
                ..default()
            },
            None,
        );
        commands
            .entity(header)
            .insert((GlobalZIndex(1000), Pickable::IGNORE));
        let right = spawn_node(
            commands,
            header,
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(8),
                ..default()
            },
            None,
        );
        self.add_labeled_icon_button(
            commands,
            right,
            UiAction::Navigation(NavigationUiAction::ToggleShop),
            self.assets.shop.icon.clone(),
            CozyButtonVariant::Neutral,
            "商店",
        );
        self.add_labeled_icon_button(
            commands,
            right,
            UiAction::Navigation(NavigationUiAction::ToggleSettings),
            self.assets.home.settings_icon.clone(),
            CozyButtonVariant::Neutral,
            "设置",
        );
        self.add_labeled_icon_button(
            commands,
            right,
            UiAction::Lobby(LobbyUiAction::LeaveRoom),
            self.assets.home.exit_icon.clone(),
            CozyButtonVariant::Danger,
            "退出",
        );
        ProfileEntry {
            form: self.form,
            assets: self.assets,
            avatars: self.avatars,
        }
        .render(commands, right, 38.0, 15.0);
    }

    fn add_labeled_icon_button(
        &self,
        commands: &mut Commands,
        parent: Entity,
        action: UiAction,
        icon: Handle<Image>,
        variant: CozyButtonVariant,
        label: &str,
    ) {
        let column = spawn_node(
            commands,
            parent,
            Node {
                width: px(44),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(1),
                ..default()
            },
            None,
        );
        add_cozy_icon_button(commands, column, action, self.assets, icon, variant);
        add_text(commands, column, label, 12.0, TEXT, self.assets);
    }
}
