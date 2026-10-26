use super::super::actions::SakuraUiAction;
use super::super::input::{SakuraInput, SakuraScroll};
use super::page::SakuraPage;
use crate::app::presentation::{MUTED, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyButtonVariant, UiAction, add_cozy_button_variant, add_cozy_disabled_button, add_cozy_panel,
};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_ui::{Scrollbar, ScrollbarSkin, TextInput, UiComposer, spawn_node};

impl SakuraPage<'_> {
    #[expect(clippy::too_many_arguments, reason = "button styling stays explicit")]
    pub(super) fn button(
        &self,
        commands: &mut UiComposer,
        parent: Entity,
        label: &str,
        action: SakuraUiAction,
        selected: bool,
        disabled: bool,
        width: Val,
    ) {
        if disabled {
            add_cozy_disabled_button(commands, parent, label, self.assets, width, 40.0);
        } else {
            add_cozy_button_variant(
                commands,
                parent,
                label,
                UiAction::Sakura(action),
                self.assets,
                width,
                40.0,
                if selected {
                    CozyButtonVariant::Primary
                } else {
                    CozyButtonVariant::Neutral
                },
            );
        }
    }

    pub(super) fn field(&self, commands: &mut UiComposer, parent: Entity, kind: SakuraInput) {
        let (label, key, value, placeholder) = match kind {
            SakuraInput::Token => ("", "sakura.token", self.state.token.as_str(), "访问密钥"),
            SakuraInput::Name => (
                "隧道名称",
                "sakura.name",
                self.state.name.as_str(),
                "例如 我的游戏",
            ),
            SakuraInput::LocalIp => (
                "本地 IP",
                "sakura.local_ip",
                self.state.local_ip.as_str(),
                "127.0.0.1",
            ),
            SakuraInput::LocalPort => (
                "本地端口",
                "sakura.local_port",
                self.state.local_port.as_str(),
                "1 到 65535",
            ),
            SakuraInput::Remote => (
                "远程端口或域名",
                "sakura.remote",
                self.state.remote.as_str(),
                "TCP/UDP 可留空随机分配",
            ),
        };
        if !label.is_empty() {
            add_text(commands, parent, label, 14.0, MUTED, self.assets);
        }
        let mut input = TextInput::new(key, value);
        input.placeholder = placeholder;
        let password = matches!(kind, SakuraInput::Token);
        input.password = password;
        input.max_characters = if password { 256 } else { 100 };
        input.font_size = if password { 18.0 } else { 15.0 };
        let editor = input.spawn(
            commands,
            parent,
            Node {
                width: percent(100),
                height: px(if password { 52 } else { 40 }),
                padding: UiRect::axes(px(12), px(8)),
                ..default()
            },
            self.assets.text_input_style(),
        );
        commands.entity(editor).insert(kind);
    }
}

pub(super) fn panel(
    commands: &mut UiComposer,
    parent: Entity,
    grow: f32,
    assets: &UiAssets,
) -> Entity {
    add_cozy_panel(
        commands,
        parent,
        Node {
            min_width: px(0),
            min_height: px(0),
            flex_basis: px(0),
            flex_grow: grow,
            padding: UiRect::all(px(16)),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            ..default()
        },
        assets,
    )
}

pub(super) fn row(commands: &mut UiComposer, parent: Entity, gap: f32) -> Entity {
    spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            align_items: AlignItems::Center,
            column_gap: px(gap),
            ..default()
        },
        None,
    )
}

pub(super) fn scroll_list(
    commands: &mut UiComposer,
    parent: Entity,
    key: &str,
    kind: SakuraScroll,
    scroll_y: f32,
) -> Entity {
    let viewport = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(0),
            flex_grow: 1.0,
            flex_basis: px(0),
            column_gap: px(8),
            ..default()
        },
        None,
    );
    let list = commands.keyed_child(
        viewport,
        key,
        (
            Node {
                min_width: px(0),
                min_height: px(0),
                flex_basis: px(0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            kind,
            RelativeCursorPosition::default(),
            ScrollPosition(Vec2::new(0.0, scroll_y)),
        ),
    );
    Scrollbar::spawn(
        commands,
        viewport,
        list,
        ScrollbarSkin::Colored(Color::srgb(0.70, 0.75, 0.87)),
    );
    list
}
