use super::super::assets::EntryIcons;
use super::super::view::GuessTable;
use super::visuals::{RevealPanel, RevealPixel};
use crate::app::presentation::{ACCENT, MUTED, TEXT, add_text};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_minecraft_guess::GuessMode;
use leocard_ui::{UiComposer, spawn_node};

impl GuessTable<'_> {
    pub(in super::super) fn answer(
        &self,
        commands: &mut UiComposer,
        heading: Entity,
        page: Entity,
    ) {
        let progress = self.game.progress;
        let round_label = add_text(
            commands,
            heading,
            match progress.mode() {
                GuessMode::TurnBased => format!(
                    "第 {}/{} 轮  第 {}/{} 局",
                    progress.current_round(),
                    progress.round_count().expect("rotating match"),
                    progress.game_index + 1,
                    progress.total_games(),
                ),
                GuessMode::Graph => "关系谜图".to_owned(),
                GuessMode::Race => format!(
                    "抢答竞速  第 {}/{} 局",
                    progress.game_index + 1,
                    progress.total_games()
                ),
            },
            14.0,
            MUTED,
            self.assets,
        );
        commands.entity(round_label).insert(Node {
            position_type: PositionType::Absolute,
            left: px(0),
            ..default()
        });
        let placeholder = spawn_node(
            commands,
            heading,
            Node {
                align_items: AlignItems::Center,
                column_gap: px(10),
                ..default()
            },
            None,
        );
        commands
            .entity(placeholder)
            .insert(RevealPanel::Placeholder);
        add_text(
            commands,
            placeholder,
            format!(
                "{}：{}",
                self.game.round.kind.label(),
                "_ ".repeat(usize::from(self.game.round.character_count))
                    .trim_end()
            ),
            23.0,
            TEXT,
            self.assets,
        );
        add_text(
            commands,
            placeholder,
            format!("({})", self.game.round.character_count),
            17.0,
            MUTED,
            self.assets,
        );
        let Some(answer) = &self.game.round.answer else {
            return;
        };
        let revealed = spawn_node(
            commands,
            heading,
            Node {
                position_type: PositionType::Absolute,
                align_items: AlignItems::Center,
                column_gap: px(10),
                ..default()
            },
            None,
        );
        commands.entity(revealed).insert(RevealPanel::Heading);
        EntryIcons::spawn(commands, revealed, self.game_assets.icon(answer), 36.0);
        add_text(
            commands,
            revealed,
            format!("{}：{}", answer.kind.label(), answer.name),
            25.0,
            ACCENT,
            self.assets,
        );
        self.answer_banner(commands, page);
    }

    fn answer_banner(&self, commands: &mut UiComposer, page: Entity) {
        let answer = self.game.round.answer.as_ref().expect("revealed answer");
        let banner = spawn_node(
            commands,
            page,
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: percent(34),
                max_width: percent(80),
                min_width: px(360),
                padding: UiRect::axes(px(26), px(20)),
                align_items: AlignItems::Center,
                column_gap: px(22),
                ..default()
            },
            None,
        );
        commands.entity(banner).insert((
            self.game_assets.popup_frame(),
            RevealPanel::Banner,
            UiTransform::from_translation(Val2::new(percent(-50), px(0))),
            ZIndex(10),
            Pickable::IGNORE,
        ));
        EntryIcons::spawn(commands, banner, self.game_assets.icon(answer), 96.0);
        let labels = spawn_node(
            commands,
            banner,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                max_width: px(330),
                ..default()
            },
            None,
        );
        add_text(commands, labels, &answer.name, 30.0, ACCENT, self.assets);
        let winner = self
            .game
            .round
            .winner
            .and_then(|id| self.game.players.iter().find(|player| player.id.0 == id));
        let caption = winner.map_or_else(
            || "答案揭晓".to_owned(),
            |player| {
                if player.id == self.game.you {
                    "你猜中了！".to_owned()
                } else {
                    format!("{} 猜中了！", player.name)
                }
            },
        );
        add_text(commands, labels, caption, 18.0, TEXT, self.assets);
        if winner.is_some() {
            for i in 0..10 {
                let sign = if i % 2 == 0 { -1.0 } else { 1.0 };
                let velocity = Vec2::new(
                    sign * (42.0 + (i / 2) as f32 * 14.0),
                    -25.0 - (i % 3) as f32 * 18.0,
                );
                commands.child(
                    banner,
                    (
                        Node {
                            position_type: PositionType::Absolute,
                            left: percent(50),
                            top: percent(50),
                            width: px(4),
                            height: px(4),
                            ..default()
                        },
                        BackgroundColor(ACCENT),
                        RevealPixel {
                            velocity,
                            delay: i as f32 * 0.02,
                        },
                        Pickable::IGNORE,
                    ),
                );
            }
        }
    }
}
