use super::super::{ProfileStat, reference_level, reference_level_index};
use crate::app::presentation::{MUTED, TEXT, add_avatar, add_coin_balance, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_protocol::{PlayerGameProfiles, PlayerGender, PlayerInteractionKind};

pub(super) struct ProfileIdentity<'a> {
    player_name: &'a str,
    gender: PlayerGender,
    avatar: Option<&'a Handle<Image>>,
    reference_points: i32,
    completed_games: u32,
    game_profiles: &'a PlayerGameProfiles,
    assets: &'a UiAssets,
}

impl<'a> ProfileIdentity<'a> {
    pub(super) fn new(
        player_name: &'a str,
        gender: PlayerGender,
        avatar: Option<&'a Handle<Image>>,
        reference_points: i32,
        completed_games: u32,
        game_profiles: &'a PlayerGameProfiles,
        assets: &'a UiAssets,
    ) -> Self {
        Self {
            player_name,
            gender,
            avatar,
            reference_points,
            completed_games,
            game_profiles,
            assets,
        }
    }

    pub(super) fn render(self, commands: &mut Commands, parent: Entity) {
        let identity = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                min_height: px(128),
                padding: UiRect::vertical(px(10)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(16),
                ..default()
            },
            None,
        );

        let avatar_area = spawn_node(
            commands,
            identity,
            Node {
                width: px(88),
                min_width: px(88),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(6),
                ..default()
            },
            None,
        );
        add_avatar(
            commands,
            avatar_area,
            self.player_name,
            self.avatar,
            86.0,
            self.assets,
        );

        let identity_text = spawn_node(
            commands,
            identity,
            Node {
                min_width: px(0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                row_gap: px(7),
                ..default()
            },
            None,
        );
        let name_row = spawn_node(
            commands,
            identity_text,
            Node {
                min_width: px(0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(8),
                ..default()
            },
            None,
        );
        add_text(
            commands,
            name_row,
            self.player_name,
            27.0,
            TEXT,
            self.assets,
        );
        let (symbol, color) = match self.gender {
            PlayerGender::Male => ("♂", Color::srgb(0.42, 0.70, 1.0)),
            PlayerGender::Female => ("♀", Color::srgb(1.0, 0.58, 0.76)),
        };
        add_text(commands, name_row, symbol, 25.0, color, self.assets);
        self.add_achievement_totals(commands, identity_text);
        self.add_interaction_totals(commands, identity_text);

        let stats = spawn_node(
            commands,
            identity,
            Node {
                width: px(324),
                min_width: px(324),
                flex_direction: FlexDirection::Row,
                column_gap: px(6),
                ..default()
            },
            None,
        );
        self.add_stat(commands, stats, "分数", self.reference_points.to_string());
        self.add_stat(
            commands,
            stats,
            "完成对局",
            self.completed_games.to_string(),
        );
        self.add_level_stat(commands, stats);
    }

    fn add_achievement_totals(&self, commands: &mut Commands, parent: Entity) {
        let row = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                min_height: px(28),
                align_items: AlignItems::Center,
                column_gap: px(18),
                ..default()
            },
            None,
        );
        for (index, count) in self
            .game_profiles
            .achievements
            .by_tier()
            .into_iter()
            .enumerate()
            .rev()
        {
            let item = spawn_node(
                commands,
                row,
                Node {
                    align_items: AlignItems::Center,
                    column_gap: px(5),
                    ..default()
                },
                None,
            );
            let icon = commands
                .spawn((
                    Node {
                        width: px(24),
                        height: px(24),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    ImageNode::new(self.assets.achievements.medals[index].clone()),
                    Pickable::IGNORE,
                ))
                .id();
            commands.entity(item).add_child(icon);
            add_text(
                commands,
                item,
                count.to_string(),
                16.0,
                Color::srgb(0.79, 0.75, 1.0),
                self.assets,
            );
        }
    }

    fn add_interaction_totals(&self, commands: &mut Commands, parent: Entity) {
        let stats = self.game_profiles.interactions.clone().unwrap_or_default();
        let row = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                min_height: px(28),
                align_items: AlignItems::Center,
                column_gap: px(18),
                ..default()
            },
            None,
        );
        add_coin_balance(
            commands,
            row,
            self.game_profiles.coins,
            24.0,
            16.0,
            self.assets,
        );
        for (kind, count) in [
            (PlayerInteractionKind::Flower, stats.flowers_received),
            (PlayerInteractionKind::Egg, stats.eggs_received),
        ] {
            let item = spawn_node(
                commands,
                row,
                Node {
                    align_items: AlignItems::Center,
                    column_gap: px(5),
                    ..default()
                },
                None,
            );
            let icon = commands
                .spawn((
                    Node {
                        width: px(24),
                        height: px(24),
                        ..default()
                    },
                    ImageNode::new(
                        self.assets
                            .social
                            .interaction_images
                            .get(&(kind, false))
                            .cloned()
                            .unwrap_or_default(),
                    ),
                    Pickable::IGNORE,
                ))
                .id();
            commands.entity(item).add_child(icon);
            add_text(
                commands,
                item,
                count.to_string(),
                16.0,
                Color::srgb(0.79, 0.75, 1.0),
                self.assets,
            );
        }
    }

    fn add_stat(
        &self,
        commands: &mut Commands,
        parent: Entity,
        label: &str,
        value: impl Into<String>,
    ) {
        let card = self.add_stat_card(commands, parent, label);
        add_text(
            commands,
            card,
            value,
            21.0,
            Color::srgb(0.79, 0.75, 1.0),
            self.assets,
        );
    }

    fn add_level_stat(&self, commands: &mut Commands, parent: Entity) {
        let card = self.add_stat_card(commands, parent, "等级");
        let row = spawn_node(
            commands,
            card,
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(2),
                ..default()
            },
            None,
        );
        let icon = commands
            .spawn((
                Node {
                    width: px(22),
                    height: px(22),
                    ..default()
                },
                ImageNode::new(
                    self.assets.home.reference_level_icons
                        [reference_level_index(self.reference_points)]
                    .clone(),
                ),
                Pickable::IGNORE,
            ))
            .id();
        commands.entity(row).add_child(icon);
        let level = reference_level(self.reference_points);
        add_text(
            commands,
            row,
            level,
            if level.chars().count() == 4 {
                18.0
            } else {
                21.0
            },
            Color::srgb(0.79, 0.75, 1.0),
            self.assets,
        );
    }

    fn add_stat_card(&self, commands: &mut Commands, parent: Entity, label: &str) -> Entity {
        let card = spawn_node(
            commands,
            parent,
            Node {
                min_width: px(0),
                min_height: px(72),
                flex_basis: px(0),
                flex_grow: 1.0,
                padding: UiRect::axes(px(4), px(10)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(4),
                ..default()
            },
            None,
        );
        commands.entity(card).insert(ProfileStat);
        add_text(commands, card, label, 12.0, MUTED, self.assets);
        card
    }
}
