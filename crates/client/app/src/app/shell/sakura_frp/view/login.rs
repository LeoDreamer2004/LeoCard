use super::super::actions::SakuraUiAction;
use super::super::input::SakuraInput;
use super::page::SakuraPage;
use crate::app::presentation::{TEXT, UiComposer, add_avatar, add_text, spawn_node};
use crate::app::shell::{PageTransitionElement, UiAction, add_cozy_checkbox, add_cozy_panel};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

impl SakuraPage<'_> {
    pub(super) fn render_login(&self, commands: &mut UiComposer, page: Entity) {
        let center = spawn_node(
            commands,
            page,
            Node {
                width: percent(100),
                flex_grow: 1.0,
                min_height: px(0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            None,
        );
        let panel = add_cozy_panel(
            commands,
            center,
            Node {
                width: px(560),
                max_width: percent(96),
                padding: UiRect::all(px(36)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(22),
                ..default()
            },
            self.assets,
        );
        commands
            .entity(panel)
            .insert((PageTransitionElement::right(1), UiTransform::IDENTITY));
        let identity = spawn_node(
            commands,
            panel,
            Node {
                width: percent(100),
                height: px(142),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(12),
                ..default()
            },
            None,
        );
        if let Some(saved) = &self.state.saved_login
            && self.state.remember_token
            && self.state.token.trim() == saved.token
        {
            add_avatar(
                commands,
                identity,
                &saved.account.name,
                self.state.avatar.as_ref(),
                88.0,
                self.assets,
            );
            add_text(
                commands,
                identity,
                &saved.account.name,
                24.0,
                TEXT,
                self.assets,
            );
        } else {
            add_text(commands, identity, "访问密钥登录", 26.0, TEXT, self.assets);
        }
        self.field(commands, panel, SakuraInput::Token);
        let options = spawn_node(
            commands,
            panel,
            Node {
                width: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: px(12),
                ..default()
            },
            None,
        );
        let links = spawn_node(
            commands,
            options,
            Node {
                align_items: AlignItems::Center,
                column_gap: px(8),
                ..default()
            },
            None,
        );
        self.link(commands, links, "注册账号", SakuraUiAction::Register);
        add_text(commands, links, "|", 15.0, TEXT, self.assets);
        self.link(commands, links, "找回密码", SakuraUiAction::RecoverPassword);
        add_cozy_checkbox(
            commands,
            options,
            "记住密码",
            self.state.remember_token,
            UiAction::Sakura(SakuraUiAction::ToggleRemember),
            self.assets,
        );
        self.button(
            commands,
            panel,
            if self.state.busy {
                "登录中…"
            } else {
                "登录"
            },
            SakuraUiAction::Login,
            true,
            self.state.busy || self.state.token.trim().is_empty(),
            percent(100),
        );
    }

    fn link(&self, commands: &mut UiComposer, parent: Entity, label: &str, action: SakuraUiAction) {
        let link = commands.child(
            parent,
            (
                Button,
                UiAction::Sakura(action),
                Node {
                    height: px(30),
                    align_items: AlignItems::Center,
                    ..default()
                },
            ),
        );
        let text = add_text(
            commands,
            link,
            label,
            15.0,
            Color::srgb(0.76, 0.72, 0.94),
            self.assets,
        );
        commands.entity(text).insert(Pickable::IGNORE);
    }
}
