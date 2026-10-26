use super::super::actions::SakuraUiAction;
use super::super::input::SakuraScroll;
use super::page::SakuraPage;
use super::widgets::{panel, row, scroll_list};
use crate::app::presentation::{MUTED, TEXT, add_text, card_background_image};
use crate::app::shell::{PageTransitionElement, UiAction, add_cozy_button, add_cozy_switch};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_sakura_frp::Tunnel;
use leocard_ui::UiComposer;

impl SakuraPage<'_> {
    pub(super) fn render_tunnels(&self, commands: &mut UiComposer, body: Entity) {
        let panel = panel(commands, body, 0.4, self.assets);
        commands
            .entity(panel)
            .insert((PageTransitionElement::left(2), UiTransform::IDENTITY));
        add_text(
            commands,
            panel,
            format!("我的隧道  {}", self.state.tunnels.len()),
            22.0,
            TEXT,
            self.assets,
        );
        let list = scroll_list(
            commands,
            panel,
            "sakura-tunnels",
            SakuraScroll::Tunnels,
            self.state.tunnel_scroll_y,
        );
        if self.state.tunnels.is_empty() {
            add_text(
                commands,
                list,
                "还没有隧道。先在右侧选择协议、节点和本地端口。",
                15.0,
                MUTED,
                self.assets,
            );
        }
        for tunnel in &self.state.tunnels {
            self.render_tunnel(commands, list, tunnel);
        }
    }

    fn render_tunnel(&self, commands: &mut UiComposer, list: Entity, tunnel: &Tunnel) {
        let card = commands.keyed_child(
            list,
            format!("tunnel:{}", tunnel.id),
            (
                Node {
                    width: percent(100),
                    flex_shrink: 0.0,
                    padding: UiRect::axes(px(18), px(16)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10),
                    ..default()
                },
                card_background_image(self.assets.home.game_card.clone(), 1.0),
            ),
        );
        let heading = row(commands, card, 12.0);
        let title = add_text(commands, heading, &tunnel.name, 22.0, TEXT, self.assets);
        commands.entity(title).insert(Node {
            flex_grow: 1.0,
            flex_basis: px(0),
            min_width: px(0),
            ..default()
        });
        let running = self.state.running.contains_key(&tunnel.id);
        let enabled = running || (!self.state.busy && tunnel.status == 0);
        add_cozy_switch(
            commands,
            heading,
            running,
            enabled.then_some(UiAction::Sakura(if running {
                SakuraUiAction::Stop(tunnel.id)
            } else {
                SakuraUiAction::Start(tunnel.id)
            })),
            self.assets,
        );
        self.render_delete(commands, heading, tunnel.id);
        let address_row = row(commands, card, 10.0);
        let address = add_text(
            commands,
            address_row,
            format!(
                "{}:{} -> {}",
                tunnel.local_ip,
                tunnel.local_port,
                tunnel.address(&self.state.nodes)
            ),
            15.0,
            TEXT,
            self.assets,
        );
        commands.entity(address).insert(Node {
            flex_grow: 1.0,
            flex_basis: px(0),
            min_width: px(0),
            ..default()
        });
        let copied = self
            .state
            .copied_address
            .is_some_and(|(id, _)| id == tunnel.id);
        let copy = add_cozy_button(
            commands,
            address_row,
            if copied { "已复制" } else { "复制地址" },
            UiAction::Sakura(SakuraUiAction::CopyAddress(tunnel.id)),
            self.assets,
            px(86),
            32.0,
        );
        commands.entity(copy).insert(Node {
            width: px(86),
            height: px(32),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        });
        let node = self
            .state
            .nodes
            .iter()
            .find(|node| node.id == tunnel.node)
            .map_or("未知节点", |node| node.name.as_str());
        add_text(
            commands,
            card,
            format!("#{}  {node}", tunnel.id),
            13.0,
            MUTED,
            self.assets,
        );
    }

    fn render_delete(&self, commands: &mut UiComposer, heading: Entity, id: u64) {
        let button = commands.child(
            heading,
            (Node {
                width: px(28),
                height: px(30),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },),
        );
        if !self.state.busy {
            commands
                .entity(button)
                .insert((Button, UiAction::Sakura(SakuraUiAction::AskDelete(id))));
        }
        let label = add_text(
            commands,
            button,
            "×",
            28.0,
            MUTED.with_alpha(if self.state.busy { 0.4 } else { 1.0 }),
            self.assets,
        );
        commands.entity(label).insert(Pickable::IGNORE);
    }
}
