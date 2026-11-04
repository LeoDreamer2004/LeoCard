//! 顶栏及其连接状态、设置和个人资料入口。

use crate::app::presentation::{BORDER, MUTED, add_text, add_title};
use crate::app::runtime::{AvatarImages, ClientResource, ConnectionDraft, UiAssets};
#[cfg(not(target_os = "android"))]
use crate::app::shell::SakuraUiAction;
use crate::app::shell::{
    AchievementEntry, ConnectionUiAction, LiquidKind, LiquidSurface, LobbyGameHeader,
    NavigationUiAction, PageTransitionElement, ProfileEntry, ShopEntry, UiAction,
    add_page_icon_button, can_edit_identity,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_client::ClientPhaseRef;
use leocard_ui::{CustomButtonMotion, UiComposer, spawn_node};

const PAGE_HEADER_SCALE: f32 = if cfg!(target_os = "android") {
    1.2
} else {
    1.0
};

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

    pub(in crate::app::shell) fn render(
        self,
        commands: &mut UiComposer,
        root: Entity,
        home_page: bool,
    ) {
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
                height: px(72.0 * PAGE_HEADER_SCALE),
                flex_shrink: 0.0,
                border: UiRect::bottom(px(2)),
                padding: UiRect {
                    right: px(if cfg!(target_os = "android") { 48 } else { 24 }),
                    ..UiRect::axes(px(24), px(5))
                },
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
                column_gap: px(14.0 * PAGE_HEADER_SCALE),
                ..default()
            },
            None,
        );
        add_title(
            commands,
            left,
            "LeoCard",
            25.0 * PAGE_HEADER_SCALE,
            Color::srgb(0.84, 0.81, 1.0),
            self.assets,
        );
        if let Some(host_port) = self.client.and_then(|client| client.0.model().host_port()) {
            let divider = spawn_node(
                commands,
                left,
                Node {
                    width: px(1),
                    height: px(24.0 * PAGE_HEADER_SCALE),
                    ..default()
                },
                Some(BORDER),
            );
            commands.entity(divider).insert(Pickable::IGNORE);
            add_text(
                commands,
                left,
                format!("端口 {host_port}"),
                15.0 * PAGE_HEADER_SCALE,
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
                column_gap: px(6.0 * PAGE_HEADER_SCALE),
                ..default()
            },
            None,
        );
        let editable_identity = can_edit_identity(self.client);
        let is_home = home_page && editable_identity;
        if is_home {
            let join = commands.child(
                right,
                (
                    Button,
                    UiAction::Connection(ConnectionUiAction::ToggleJoin),
                    Node {
                        width: px(146.0 * PAGE_HEADER_SCALE),
                        height: px(48.0 * PAGE_HEADER_SCALE),
                        ..default()
                    },
                    CustomButtonMotion,
                ),
            );
            LiquidSurface::mount(
                commands,
                join,
                join,
                self.assets.home.join_room.clone(),
                Color::WHITE,
                LiquidKind::JoinSweep,
                0.0,
            );
        }
        if self.client.is_none_or(|client| {
            matches!(
                client.0.model().phase(),
                ClientPhaseRef::Idle | ClientPhaseRef::Closed
            )
        }) {
            #[cfg(not(target_os = "android"))]
            add_page_icon_button(
                commands,
                right,
                UiAction::Sakura(SakuraUiAction::Show),
                self.assets,
                self.assets.home.network_icon.clone(),
                "联机",
                PAGE_HEADER_SCALE,
            );
            ShopEntry {
                assets: self.assets,
            }
            .render(commands, right, PAGE_HEADER_SCALE);
            AchievementEntry {
                assets: self.assets,
            }
            .render(commands, right, PAGE_HEADER_SCALE);
        }
        add_page_icon_button(
            commands,
            right,
            UiAction::Navigation(NavigationUiAction::ToggleSettings),
            self.assets,
            self.assets.home.settings_icon.clone(),
            "设置",
            PAGE_HEADER_SCALE,
        );
        ProfileEntry {
            form: self.form,
            assets: self.assets,
            avatars: self.avatars,
        }
        .render(
            commands,
            right,
            44.0 * PAGE_HEADER_SCALE,
            17.5 * PAGE_HEADER_SCALE,
        );
        if editable_identity {
            let menu = commands.child(
                right,
                (
                    Button,
                    UiAction::Connection(ConnectionUiAction::ToggleIdentity),
                    Node {
                        max_width: px(180),
                        min_width: px(0),
                        align_items: AlignItems::Center,
                        column_gap: px(8),
                        padding: UiRect::axes(px(6), px(10)),
                        ..default()
                    },
                ),
            );
            let name = add_text(
                commands,
                menu,
                &self.form.player_name,
                16.0 * PAGE_HEADER_SCALE,
                Color::srgb(0.88, 0.84, 1.0),
                self.assets,
            );
            commands
                .entity(name)
                .insert((Pickable::IGNORE, TextLayout::default().with_no_wrap()));
            commands.entity(name).insert(Node {
                min_width: px(0),
                overflow: Overflow::clip(),
                ..default()
            });
            let mut arrow = ImageNode::new(self.assets.achievements.scroll_arrow.clone());
            arrow.flip_y = true;
            commands.child(
                menu,
                (
                    Node {
                        width: px(13),
                        height: px(8),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    arrow,
                    Pickable::IGNORE,
                ),
            );
        }
    }
}
