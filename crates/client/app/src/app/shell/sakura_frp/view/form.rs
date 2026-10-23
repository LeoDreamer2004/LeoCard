use super::super::actions::SakuraUiAction;
use super::super::input::{SakuraInput, SakuraScroll};
use super::page::SakuraPage;
use super::widgets::{panel, row, scroll_list};
use crate::app::presentation::{MUTED, TEXT, UiComposer, add_rule_help, add_text, spawn_node};
use crate::app::shell::{
    CozyButtonVariant, CozyDropdown, CozyDropdownOption, PageTransitionElement, UiAction,
    add_cozy_button_variant, add_cozy_checkbox,
};
use bevy::prelude::*;
use leocard_sakura_frp::TunnelType;

impl SakuraPage<'_> {
    pub(super) fn render_form(&self, commands: &mut UiComposer, root: Entity, body: Entity) {
        let form = panel(commands, body, 0.6, self.assets);
        commands
            .entity(form)
            .insert((PageTransitionElement::right(2), UiTransform::IDENTITY));
        add_text(commands, form, "创建隧道", 22.0, TEXT, self.assets);
        let (name, protocol) = split_fields(commands, form, 190.0);
        self.field(commands, name, SakuraInput::Name);
        add_text(commands, protocol, "协议", 14.0, MUTED, self.assets);
        CozyDropdown {
            label: self.state.kind.label(),
            toggle_action: UiAction::Sakura(SakuraUiAction::ToggleKindMenu),
            close_action: UiAction::Sakura(SakuraUiAction::CloseKindMenu),
            options: [
                TunnelType::Tcp,
                TunnelType::Udp,
                TunnelType::Http,
                TunnelType::Https,
            ]
            .into_iter()
            .map(|kind| CozyDropdownOption {
                label: kind.label(),
                selected: self.state.kind == kind,
                action: UiAction::Sakura(SakuraUiAction::SelectKind(kind)),
            })
            .collect(),
        }
        .render(
            commands,
            root,
            protocol,
            true,
            self.state.kind_menu_open,
            self.assets,
        );
        let (local_ip, local_port) = split_fields(commands, form, 190.0);
        self.field(commands, local_ip, SakuraInput::LocalIp);
        self.field(commands, local_port, SakuraInput::LocalPort);
        self.field(commands, form, SakuraInput::Remote);
        let heading = spawn_node(
            commands,
            form,
            Node {
                width: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: px(12),
                ..default()
            },
            None,
        );
        add_text(commands, heading, "选择节点", 16.0, TEXT, self.assets);
        let public_game = spawn_node(
            commands,
            heading,
            Node {
                align_items: AlignItems::Center,
                column_gap: px(6),
                ..default()
            },
            None,
        );
        add_cozy_checkbox(
            commands,
            public_game,
            "公开游戏服务器",
            self.state.public_game,
            UiAction::Sakura(SakuraUiAction::TogglePublicGame),
            self.assets,
        );
        add_rule_help(
            commands,
            public_game,
            "公开游戏服务器只能使用专用节点。私人联机及其他应用可使用普通节点。",
            self.assets,
        );
        self.render_nodes(commands, form);
        self.button(
            commands,
            form,
            "创建隧道",
            SakuraUiAction::Create,
            true,
            self.state.busy || self.state.node.is_none(),
            percent(100),
        );
    }

    pub(super) fn render_nodes(&self, commands: &mut UiComposer, form: Entity) {
        let container = spawn_node(
            commands,
            form,
            Node {
                width: percent(100),
                height: px(168),
                min_height: px(0),
                ..default()
            },
            None,
        );
        let list = scroll_list(
            commands,
            container,
            "sakura-nodes",
            SakuraScroll::Nodes,
            self.state.node_scroll_y,
        );
        let grid = spawn_node(
            commands,
            list,
            Node {
                width: percent(100),
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::flex(2, 1.0),
                grid_auto_rows: vec![GridTrack::px(50.0)],
                column_gap: px(10),
                row_gap: px(8),
                flex_shrink: 0.0,
                ..default()
            },
            None,
        );
        let mut count = 0;
        for node in self
            .state
            .nodes
            .iter()
            .filter(|node| self.state.node_compatible(node))
        {
            count += 1;
            let label = format!("{}\n#{}  VIP {}", node.name, node.id, node.vip);
            let button = add_cozy_button_variant(
                commands,
                grid,
                &label,
                UiAction::Sakura(SakuraUiAction::SelectNode(node.id)),
                self.assets,
                percent(100),
                50.0,
                if self.state.node == Some(node.id) {
                    CozyButtonVariant::Primary
                } else {
                    CozyButtonVariant::Neutral
                },
            );
            let label = commands.entity_for(button, "label");
            commands.entity(label).insert((
                TextLayout::justify(Justify::Center),
                Node {
                    width: percent(100),
                    min_width: px(0),
                    flex_shrink: 0.0,
                    padding: UiRect::horizontal(px(8)),
                    ..default()
                },
            ));
        }
        if count == 0 {
            add_text(
                commands,
                list,
                "没有符合当前协议和用途的可用节点",
                13.0,
                MUTED,
                self.assets,
            );
        }
    }
}

fn split_fields(commands: &mut UiComposer, parent: Entity, right_width: f32) -> (Entity, Entity) {
    let row = row(commands, parent, 12.0);
    let [left, right] = [
        Node {
            flex_grow: 1.0,
            flex_basis: px(0),
            ..default()
        },
        Node {
            width: px(right_width),
            flex_shrink: 0.0,
            ..default()
        },
    ]
    .map(|layout| {
        spawn_node(
            commands,
            row,
            Node {
                min_width: px(0),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..layout
            },
            None,
        )
    });
    (left, right)
}
