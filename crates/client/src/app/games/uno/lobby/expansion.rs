use super::super::{UnoExpansionStatus, UnoExpansionStatusFrame, UnoUiAction};
use crate::app::presentation::{ButtonHighlight, TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyModalBackdrop, CozyModalKind, CozyModalPanel, UiAction, add_cozy_close_button,
    add_cozy_panel, cozy_backdrop_color, cozy_panel_transform,
};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_uno::UnoRuleSet;

pub(crate) fn render_uno_expansion_settings(
    commands: &mut Commands,
    root: Entity,
    rules: UnoRuleSet,
    can_configure: bool,
    progress: f32,
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
        Some(cozy_backdrop_color(progress)),
    );
    commands.entity(overlay).insert((
        GlobalZIndex(2100),
        FocusPolicy::Block,
        CozyModalBackdrop(CozyModalKind::UnoExpansionSettings),
    ));
    let modal = add_cozy_panel(
        commands,
        overlay,
        Node {
            width: px(540),
            max_width: percent(92),
            padding: UiRect::all(px(24)),
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            ..default()
        },
        assets,
    );
    commands.entity(modal).insert((
        CozyModalPanel(CozyModalKind::UnoExpansionSettings),
        cozy_panel_transform(progress),
    ));
    let heading = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        None,
    );
    add_text(commands, heading, "扩展包设置", 24.0, TEXT, assets);
    add_cozy_close_button(
        commands,
        heading,
        UiAction::Uno(UnoUiAction::ToggleExpansionSettings),
        assets,
    );
    spawn_node(
        commands,
        modal,
        Node {
            width: px(82),
            height: px(2),
            ..default()
        },
        Some(Color::srgb(0.64, 0.59, 0.93)),
    );
    if rules.is_classic() {
        for row in [
            UnoExpansionRow {
                name: "Swap Pack",
                enabled: rules.swap_pack,
                toggled_rules: UnoRuleSet {
                    swap_pack: !rules.swap_pack,
                    ..rules
                },
            },
            UnoExpansionRow {
                name: "Reverse Pack",
                enabled: rules.reverse_pack,
                toggled_rules: UnoRuleSet {
                    reverse_pack: !rules.reverse_pack,
                    ..rules
                },
            },
            UnoExpansionRow {
                name: "Stack Pack",
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
}

struct UnoExpansionRow {
    name: &'static str,
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
                min_height: px(56),
                padding: UiRect::axes(px(11), px(8)),
                align_items: AlignItems::Center,
                column_gap: px(10),
                border: UiRect::bottom(px(1)),
                ..default()
            },
            Some(Color::srgb(0.30, 0.31, 0.34)),
        );
        commands
            .entity(row)
            .insert(BorderColor::all(Color::srgba(0.70, 0.69, 0.77, 0.25)));
        let name_slot = spawn_node(
            commands,
            row,
            Node {
                min_width: px(0),
                flex_grow: 1.0,
                ..default()
            },
            None,
        );
        add_text(commands, name_slot, self.name, 16.0, TEXT, assets);
        self.add_status(commands, row, editable, assets);
    }

    fn add_status(
        &self,
        commands: &mut Commands,
        parent: Entity,
        editable: bool,
        assets: &UiAssets,
    ) {
        let mut image = ImageNode::new(if self.enabled {
            assets.home.checkbox_selected.clone()
        } else {
            assets.home.checkbox.clone()
        });
        if !editable {
            image.color = Color::WHITE.with_alpha(0.62);
        }
        let mut status = commands.spawn((
            UnoExpansionStatus,
            Node {
                width: px(34),
                height: px(34),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            image,
        ));
        if editable {
            status.insert((
                Button,
                UiAction::Uno(UnoUiAction::UpdateRules(self.toggled_rules)),
                UnoExpansionStatusFrame,
            ));
        }
        let status = status.id();
        commands.entity(parent).add_child(status);
        if editable {
            let hover_image = ImageNode::new(if self.enabled {
                assets.home.checkbox_selected_highlighted.clone()
            } else {
                assets.home.checkbox_highlighted.clone()
            });
            let hover = commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0),
                        right: px(0),
                        top: px(0),
                        bottom: px(0),
                        ..default()
                    },
                    hover_image,
                    Visibility::Hidden,
                    FocusPolicy::Pass,
                ))
                .id();
            commands.entity(status).add_child(hover);
            commands.entity(status).insert(ButtonHighlight {
                overlay: hover,
                arrows: None,
            });
        }
    }
}
