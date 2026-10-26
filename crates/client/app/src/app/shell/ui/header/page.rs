//! 顶栏及其连接状态、设置和个人资料入口。

use crate::app::presentation::{BORDER, MUTED, add_text};
use crate::app::runtime::{AvatarImages, ClientResource, ConnectionDraft, UiAssets};
#[cfg(not(target_os = "android"))]
use crate::app::shell::SakuraUiAction;
use crate::app::shell::{
    AchievementEntry, CozyButtonVariant, LobbyGameHeader, NavigationUiAction,
    PageTransitionElement, ProfileEntry, ShopEntry, UiAction, add_cozy_labeled_icon_button,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_client::ClientPhaseRef;
use leocard_ui::{UiComposer, spawn_node};

pub(in crate::app::shell) struct Header<'a> {
    pub(super) client: Option<&'a ClientResource>,
    pub(super) form: &'a ConnectionDraft,
    pub(super) assets: &'a UiAssets,
    pub(super) avatars: &'a AvatarImages,
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

    pub(in crate::app::shell) fn render(self, commands: &mut UiComposer, root: Entity) {
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
            #[cfg(not(target_os = "android"))]
            add_cozy_labeled_icon_button(
                commands,
                right,
                UiAction::Sakura(SakuraUiAction::Show),
                self.assets,
                self.assets.home.network_icon.clone(),
                CozyButtonVariant::Cool,
                "联机",
            );
            ShopEntry {
                assets: self.assets,
            }
            .render(commands, right);
            AchievementEntry {
                assets: self.assets,
            }
            .render(commands, right);
        }
        add_cozy_labeled_icon_button(
            commands,
            right,
            UiAction::Navigation(NavigationUiAction::ToggleSettings),
            self.assets,
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
}
