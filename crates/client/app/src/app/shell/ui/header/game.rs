use super::page::Header;
use super::state::{
    GameHeaderAvatar, GameHeaderItem, GameHeaderMotion, avatar_size, height, item_width,
    right_inset,
};
use crate::app::presentation::game_side_control_scale;
use crate::app::shell::{
    CozyButtonVariant, LobbyUiAction, NavigationUiAction, ProfileEntry, UiAction,
    add_cozy_labeled_icon_button,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_platform::is_mobile;
use leocard_ui::{UiComposer, spawn_node};

impl Header<'_> {
    pub(super) fn render_game_header(&self, commands: &mut UiComposer, root: Entity) {
        let header = commands.keyed_child(
            root,
            "game-header",
            (
                Node {
                    position_type: PositionType::Absolute,
                    right: px(0),
                    top: px(0),
                    width: percent(100),
                    height: px(height()),
                    ..default()
                },
                GlobalZIndex(1000),
                Pickable::IGNORE,
            ),
        );
        commands.entity(header).init(GameHeaderMotion::default());
        for (slot, (label, action, icon, variant)) in [
            (
                "退出",
                UiAction::Lobby(LobbyUiAction::RequestLeaveRoom),
                self.assets.home.exit_icon.clone(),
                CozyButtonVariant::Danger,
            ),
            (
                "设置",
                UiAction::Navigation(NavigationUiAction::ToggleSettings),
                self.assets.home.settings_icon.clone(),
                CozyButtonVariant::Neutral,
            ),
            (
                "商店",
                UiAction::Navigation(NavigationUiAction::ToggleShop),
                self.assets.shop.icon.clone(),
                CozyButtonVariant::Neutral,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let item = GameHeaderItem {
                slot,
                applied: None,
            };
            let panel = spawn_node(
                commands,
                header,
                Node {
                    position_type: PositionType::Absolute,
                    right: px(item.right()),
                    top: px(4.0 * game_side_control_scale()),
                    width: px(item_width()),
                    height: px(height() - 8.0 * game_side_control_scale()),
                    display: Display::None,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                None,
            );
            commands
                .entity(panel)
                .insert((item, UiTransform::IDENTITY, Pickable::IGNORE));
            add_cozy_labeled_icon_button(
                commands,
                panel,
                action,
                self.assets,
                icon,
                variant,
                label,
                game_side_control_scale(),
            );
        }
        let anchor = spawn_node(
            commands,
            header,
            Node {
                position_type: PositionType::Absolute,
                right: px(right_inset()),
                top: px((height() - avatar_size()) * 0.5),
                width: px(avatar_size()),
                height: px(avatar_size()),
                ..default()
            },
            None,
        );
        commands
            .entity(anchor)
            .insert((ZIndex(1), Pickable::IGNORE));
        let avatar = ProfileEntry {
            form: self.form,
            assets: self.assets,
            avatars: self.avatars,
        }
        .render(
            commands,
            anchor,
            avatar_size(),
            15.0 * game_side_control_scale(),
        );
        commands.entity(avatar).insert(GameHeaderAvatar);
        if is_mobile() {
            commands.entity(avatar).remove::<UiAction>();
        }
    }
}
