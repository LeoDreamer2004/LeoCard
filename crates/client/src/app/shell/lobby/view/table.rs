use super::*;
use crate::app::games::lobby_rule_labels;
#[cfg(feature = "developer")]
use crate::app::presentation::MUTED;
use crate::app::presentation::{TABLE_BG, TEXT, add_text, spawn_node};
use crate::app::runtime::{ClientResource, UiAssets};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::{GameRules, LobbySnapshot};

pub(super) struct LobbyTable<'a> {
    client: &'a ClientResource,
    lobby: &'a LobbySnapshot,
    assets: &'a UiAssets,
}

impl<'a> LobbyTable<'a> {
    pub(super) fn new(
        client: &'a ClientResource,
        lobby: &'a LobbySnapshot,
        assets: &'a UiAssets,
    ) -> Self {
        Self {
            client,
            lobby,
            assets,
        }
    }

    pub(super) fn render(self, commands: &mut Commands, parent: Entity) {
        let table = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(150),
                    top: px(108),
                    width: px(320),
                    height: px(174),
                    padding: UiRect::axes(px(24), px(18)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: px(12),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(percent(50)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                ImageNode::new(self.assets.table_felt.clone())
                    .with_mode(NodeImageMode::Stretch)
                    .with_color(Color::srgba(0.72, 0.83, 0.76, 0.92)),
                BackgroundColor(TABLE_BG),
                BorderColor::all(Color::srgba(0.62, 0.82, 0.70, 0.28)),
                BoxShadow::new(Color::BLACK.with_alpha(0.38), px(2), px(7), px(0), px(9)),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(parent).add_child(table);

        let metrics = LobbyMetrics::new(self.lobby);
        add_text(
            commands,
            table,
            format!(
                "等待准备  {}/{}",
                metrics.ready_player_count(),
                metrics.connected_player_count()
            ),
            17.0,
            TEXT,
            self.assets,
        );
        #[cfg(feature = "developer")]
        if self.client.0.model().you() == self.lobby.host {
            add_text(
                commands,
                table,
                "右键空座添加机器人，右键机器人移除",
                10.5,
                MUTED,
                self.assets,
            );
        }
        #[cfg(not(feature = "developer"))]
        let _ = self.client;
        LobbyRuleSummary::new(&self.lobby.rules, self.assets).render(commands, table);
    }
}

struct LobbyRuleSummary<'a> {
    rules: &'a GameRules,
    assets: &'a UiAssets,
}

impl<'a> LobbyRuleSummary<'a> {
    fn new(rules: &'a GameRules, assets: &'a UiAssets) -> Self {
        Self { rules, assets }
    }

    fn render(self, commands: &mut Commands, parent: Entity) {
        let chips = spawn_node(
            commands,
            parent,
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(6),
                ..default()
            },
            None,
        );
        for label in lobby_rule_labels(self.rules) {
            self.add_chip(commands, chips, label);
        }
    }

    fn add_chip(&self, commands: &mut Commands, parent: Entity, label: String) {
        let chip = spawn_node(
            commands,
            parent,
            Node {
                min_height: px(25),
                padding: UiRect::axes(px(8), px(3)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            Some(Color::srgba(0.015, 0.065, 0.048, 0.72)),
        );
        commands
            .entity(chip)
            .insert(BorderColor::all(Color::srgba(0.70, 0.88, 0.78, 0.18)));
        add_text(commands, chip, label, 11.5, TEXT, self.assets);
    }
}
