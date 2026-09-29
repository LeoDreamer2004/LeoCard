use super::*;
use crate::app::games::lobby_rule_labels;
#[cfg(feature = "developer")]
use crate::app::presentation::MUTED;
use crate::app::presentation::{TEXT, add_text, spawn_node};
use crate::app::runtime::{ClientResource, UiAssets};
use bevy::prelude::*;
use bevy::ui::{BackgroundGradient, ColorStop, FocusPolicy, Gradient, LinearGradient};
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
        let rail = spawn_node(
            commands,
            parent,
            Node {
                position_type: PositionType::Absolute,
                left: px(145),
                top: px(102),
                width: px(330),
                height: px(186),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            Some(Color::srgb(0.095, 0.085, 0.13)),
        );
        commands.entity(rail).insert((
            BackgroundGradient(vec![Gradient::Linear(LinearGradient::to_bottom(vec![
                ColorStop::percent(Color::srgb(0.27, 0.25, 0.34), 0.0),
                ColorStop::percent(Color::srgb(0.11, 0.10, 0.15), 42.0),
                ColorStop::percent(Color::srgb(0.055, 0.055, 0.085), 100.0),
            ]))]),
            BorderColor::all(Color::srgba(0.72, 0.68, 0.88, 0.42)),
            BoxShadow::new(Color::BLACK.with_alpha(0.52), px(2), px(8), px(0), px(10)),
            FocusPolicy::Pass,
        ));
        let table = spawn_node(
            commands,
            rail,
            Node {
                width: px(314),
                height: px(170),
                padding: UiRect::axes(px(18), px(12)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(9),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(percent(50)),
                overflow: Overflow::clip(),
                ..default()
            },
            Some(Color::srgb(0.075, 0.085, 0.13)),
        );
        commands.entity(table).insert((
            BackgroundGradient(vec![Gradient::Linear(LinearGradient::to_bottom(vec![
                ColorStop::percent(Color::srgb(0.12, 0.13, 0.20), 0.0),
                ColorStop::percent(Color::srgb(0.065, 0.075, 0.13), 100.0),
            ]))]),
            BorderColor::all(Color::srgba(0.62, 0.58, 0.78, 0.36)),
            FocusPolicy::Pass,
        ));
        let felt = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(0),
                    bottom: px(0),
                    ..default()
                },
                ImageNode::new(self.assets.table_felt.clone())
                    .with_mode(NodeImageMode::Stretch)
                    .with_color(Color::srgba(0.47, 0.43, 0.70, 0.10)),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(table).add_child(felt);

        let metrics = LobbyMetrics::new(self.lobby);
        add_text(
            commands,
            table,
            format!(
                "等待准备  {}/{}",
                metrics.ready_player_count(),
                metrics.connected_player_count()
            ),
            21.0,
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
        spawn_node(
            commands,
            table,
            Node {
                width: px(86),
                height: px(1),
                ..default()
            },
            Some(Color::srgba(0.72, 0.68, 0.87, 0.42)),
        );
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
                width: percent(100),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(6),
                row_gap: px(4),
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
                border_radius: BorderRadius::all(px(4)),
                ..default()
            },
            Some(Color::srgba(0.18, 0.17, 0.25, 0.72)),
        );
        commands
            .entity(chip)
            .insert(BorderColor::all(Color::srgba(0.72, 0.68, 0.87, 0.28)));
        add_text(commands, chip, label, 11.5, TEXT, self.assets);
    }
}
