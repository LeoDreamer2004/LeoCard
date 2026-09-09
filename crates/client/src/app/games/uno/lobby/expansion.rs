use super::super::{UnoExpansionStatus, UnoExpansionStatusFrame, UnoUiAction};
use crate::app::presentation::{
    BORDER, ButtonKind, DANGER, HEADER_BG, MUTED, PANEL, PANEL_ALT, PanelSkin, READY, TEXT,
    add_action_button, add_panel, add_section_title, add_text, spawn_node,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_uno::UnoRuleSet;

pub(crate) fn render_uno_expansion_settings(
    commands: &mut Commands,
    root: Entity,
    rules: UnoRuleSet,
    can_configure: bool,
    assets: &UiAssets,
) {
    let overlay = spawn_node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.62)),
    );
    commands
        .entity(overlay)
        .insert((GlobalZIndex(2100), FocusPolicy::Block));
    let modal = add_panel(
        commands,
        overlay,
        Node {
            width: px(620),
            max_width: percent(92),
            flex_direction: FlexDirection::Column,
            row_gap: px(14),
            ..default()
        },
        PANEL,
        PanelSkin::Window,
        assets,
    );
    add_section_title(commands, modal, "扩展包设置", assets);
    add_text(
        commands,
        modal,
        if rules.is_no_mercy() {
            "No Mercy 使用独立的扩展包设置。"
        } else if rules.is_flip() {
            "UNO FLIP 使用独立的扩展包设置。"
        } else {
            "选择要加入本房间牌堆的可选扩展包。"
        },
        13.0,
        MUTED,
        assets,
    );
    if rules.is_classic() {
        for row in [
            UnoExpansionRow {
                name: "Swap Pack",
                description: "以交换手牌为特色，你的手牌随时可能变成别人的",
                enabled: rules.swap_pack,
                toggled_rules: UnoRuleSet {
                    swap_pack: !rules.swap_pack,
                    ..rules
                },
            },
            UnoExpansionRow {
                name: "Reverse Pack",
                description: "以改变方向为特色，小心罚牌反弹——你可能会被自己罚到！",
                enabled: rules.reverse_pack,
                toggled_rules: UnoRuleSet {
                    reverse_pack: !rules.reverse_pack,
                    ..rules
                },
            },
            UnoExpansionRow {
                name: "Stack Pack",
                description: "以累计罚牌为特色，加入堆叠 +1、+2、万能 +3 与随机堆叠牌",
                enabled: rules.stack_pack,
                toggled_rules: UnoRuleSet {
                    stack_pack: !rules.stack_pack,
                    ..rules
                },
            },
        ] {
            row.render(commands, modal, can_configure, assets);
        }
    } else {
        add_text(
            commands,
            modal,
            if rules.is_flip() {
                "当前尚未加入 UNO FLIP 扩展包。"
            } else {
                "当前尚未加入 No Mercy 扩展包。"
            },
            15.0,
            TEXT,
            assets,
        );
    }
    let actions = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            justify_content: JustifyContent::FlexEnd,
            ..default()
        },
        None,
    );
    add_action_button(
        commands,
        actions,
        "关闭",
        UiAction::Uno(UnoUiAction::ToggleExpansionSettings),
        ButtonKind::Secondary,
        assets,
    );
}

struct UnoExpansionRow {
    name: &'static str,
    description: &'static str,
    enabled: bool,
    toggled_rules: UnoRuleSet,
}

impl UnoExpansionRow {
    fn render(self, commands: &mut Commands, parent: Entity, editable: bool, assets: &UiAssets) {
        let row = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                min_height: px(86),
                padding: UiRect::all(px(13)),
                align_items: AlignItems::Center,
                column_gap: px(14),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(8)),
                ..default()
            },
            Some(PANEL_ALT.with_alpha(0.86)),
        );
        commands.entity(row).insert(BorderColor::all(BORDER));
        let name_slot = spawn_node(
            commands,
            row,
            Node {
                width: px(112),
                flex_shrink: 0.0,
                ..default()
            },
            None,
        );
        add_text(commands, name_slot, self.name, 16.0, TEXT, assets);
        self.add_status(commands, row, editable, assets);
        let description_slot = spawn_node(
            commands,
            row,
            Node {
                min_width: px(0),
                flex_grow: 1.0,
                ..default()
            },
            None,
        );
        add_text(
            commands,
            description_slot,
            self.description,
            13.0,
            MUTED,
            assets,
        );
    }

    fn add_status(
        &self,
        commands: &mut Commands,
        parent: Entity,
        editable: bool,
        assets: &UiAssets,
    ) {
        let mut status = commands.spawn((
            UnoExpansionStatus,
            Node {
                width: px(38),
                height: px(38),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ));
        if editable {
            status.insert((
                Button,
                UiAction::Uno(UnoUiAction::UpdateRules(self.toggled_rules)),
                UnoExpansionStatusFrame,
                BorderColor::all(if self.enabled {
                    READY.with_alpha(0.82)
                } else {
                    DANGER.with_alpha(0.82)
                }),
                BackgroundColor(HEADER_BG.with_alpha(0.92)),
                Node {
                    width: px(38),
                    height: px(38),
                    flex_shrink: 0.0,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(6)),
                    ..default()
                },
            ));
        }
        let status = status.id();
        commands.entity(parent).add_child(status);
        add_text(
            commands,
            status,
            if self.enabled { "✓" } else { "×" },
            24.0,
            if self.enabled { READY } else { DANGER },
            assets,
        );
    }
}
