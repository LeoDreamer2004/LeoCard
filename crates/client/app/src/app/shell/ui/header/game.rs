use super::page::Header;
use super::state::{
    AVATAR_SIZE, GameHeaderAvatar, GameHeaderItem, GameHeaderMotion, HEIGHT, ITEM_WIDTH,
    RIGHT_INSET,
};
use crate::app::presentation::GAME_SIDE_CONTROL_SCALE;
use crate::app::shell::{
    CozyButtonVariant, LobbyUiAction, NavigationUiAction, ProfileEntry, UiAction,
    add_cozy_labeled_icon_button,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
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
                    height: px(HEIGHT),
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
                    top: px(4.0 * GAME_SIDE_CONTROL_SCALE),
                    width: px(ITEM_WIDTH),
                    height: px(HEIGHT - 8.0 * GAME_SIDE_CONTROL_SCALE),
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
            );
        }
        let anchor = spawn_node(
            commands,
            header,
            Node {
                position_type: PositionType::Absolute,
                right: px(RIGHT_INSET),
                top: px((HEIGHT - AVATAR_SIZE) * 0.5),
                width: px(AVATAR_SIZE),
                height: px(AVATAR_SIZE),
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
            AVATAR_SIZE,
            15.0 * GAME_SIDE_CONTROL_SCALE,
        );
        commands.entity(avatar).insert(GameHeaderAvatar);
        #[cfg(target_os = "android")]
        commands.entity(avatar).remove::<UiAction>();
    }
}
