use super::super::actions::SakuraUiAction;
use super::page::SakuraPage;
use crate::app::presentation::{
    AnchoredPopup, BORDER, MUTED, TEXT, UiComposer, add_avatar, add_text, spawn_node,
};
use crate::app::shell::{
    CozyButtonVariant, PageMotion, UiAction, add_cozy_button_variant, add_cozy_disabled_button,
    add_cozy_panel,
};
use bevy::picking::Pickable;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(Component)]
pub(in super::super) struct AccountMenu {
    avatar: Entity,
    close_delay: f32,
}

impl SakuraPage<'_> {
    pub(super) fn render_account(&self, commands: &mut UiComposer, heading: Entity) {
        let Some(user) = &self.state.user else {
            return;
        };
        let account = spawn_node(
            commands,
            heading,
            Node {
                align_items: AlignItems::Center,
                column_gap: px(12),
                flex_shrink: 0.0,
                ..default()
            },
            None,
        );
        add_text(commands, account, &user.name, 17.0, TEXT, self.assets);
        let avatar = add_avatar(
            commands,
            account,
            &user.name,
            self.state.avatar.as_ref(),
            44.0,
            self.assets,
        );
        commands
            .entity(avatar)
            .insert((Pickable::default(), PickingInteraction::default()));
        let menu = add_cozy_panel(
            commands,
            avatar,
            Node {
                position_type: PositionType::Absolute,
                width: px(300),
                padding: UiRect::all(px(18)),
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            },
            self.assets,
        );
        commands.entity(menu).insert((
            AccountMenu {
                avatar,
                close_delay: 0.0,
            },
            AnchoredPopup(avatar),
            GlobalZIndex(1500),
            Visibility::Hidden,
            Pickable::default(),
        ));
        add_text(commands, menu, &user.name, 23.0, TEXT, self.assets);
        spawn_node(
            commands,
            menu,
            Node {
                width: percent(100),
                height: px(1),
                flex_shrink: 0.0,
                ..default()
            },
            Some(BORDER),
        );
        let details = spawn_node(
            commands,
            menu,
            Node {
                width: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: px(12),
                ..default()
            },
            None,
        );
        add_text(
            commands,
            details,
            format!("UID {}", user.id),
            14.0,
            MUTED,
            self.assets,
        );
        let group = user
            .group
            .as_ref()
            .map_or("普通用户", |group| group.name.as_str());
        add_text(commands, details, group, 14.0, MUTED, self.assets);
        for (label, action, variant) in [
            (
                "刷新状态",
                SakuraUiAction::Refresh,
                CozyButtonVariant::Neutral,
            ),
            (
                "退出登录",
                SakuraUiAction::Logout,
                CozyButtonVariant::Danger,
            ),
        ] {
            if self.state.busy {
                add_cozy_disabled_button(commands, menu, label, self.assets, percent(100), 40.0);
            } else {
                add_cozy_button_variant(
                    commands,
                    menu,
                    label,
                    UiAction::Sakura(action),
                    self.assets,
                    percent(100),
                    40.0,
                    variant,
                );
            }
        }
    }
}

pub(in super::super) fn sync_account_menu(
    time: Res<Time>,
    motion: Res<PageMotion>,
    windows: Query<&Window, With<PrimaryWindow>>,
    avatars: Query<(&PickingInteraction, &InheritedVisibility)>,
    mut menus: Query<(
        &mut AccountMenu,
        &ComputedNode,
        &UiGlobalTransform,
        &mut Visibility,
    )>,
) {
    let cursor = windows
        .single()
        .ok()
        .filter(|window| window.focused)
        .and_then(Window::physical_cursor_position);
    for (mut menu, node, transform, mut visibility) in &mut menus {
        let can_show = !motion.active()
            && cursor.is_some()
            && avatars
                .get(menu.avatar)
                .is_ok_and(|(_, visible)| visible.get());
        let hovered = can_show
            && (avatars
                .get(menu.avatar)
                .is_ok_and(|(interaction, _)| *interaction != PickingInteraction::None)
                || *visibility == Visibility::Visible
                    && cursor.is_some_and(|point| node.contains_point(*transform, point)));
        menu.close_delay = if !can_show {
            0.0
        } else if hovered {
            0.18
        } else {
            (menu.close_delay - time.delta_secs()).max(0.0)
        };
        visibility.set_if_neq(if menu.close_delay > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    }
}
