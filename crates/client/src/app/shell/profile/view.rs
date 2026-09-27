//! 玩家档案弹窗、游戏标签和互动统计视图。

use super::super::{
    CozyModalBackdrop, CozyModalKind, CozyModalPanel, NavigationUiAction, UiAction,
    add_cozy_close_button, add_cozy_panel, cozy_backdrop_color, cozy_panel_transform,
};
use super::{
    ProfileGameColumn, ProfileGameContent, ProfileGameTab, ProfileGameTabButton, ProfileStat,
    SelectedProfileGameTab, TAB_IDLE, mahjong_profile_rows, qigui523_profile_rows, reference_level,
    reference_level_index, shengji_profile_rows, texas_holdem_profile_rows, uno_profile_rows,
};
use crate::app::presentation::{MUTED, TEXT, add_avatar, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::{PlayerGameProfiles, PlayerInteractionKind, PlayerInteractionStats};

pub(crate) struct ProfileModal<'a> {
    player_name: &'a str,
    avatar: Option<&'a Handle<Image>>,
    reference_points: i32,
    completed_games: u32,
    game_profiles: &'a PlayerGameProfiles,
    selected_game: ProfileGameTab,
    assets: &'a UiAssets,
}

impl<'a> ProfileModal<'a> {
    pub(crate) fn new(
        player_name: &'a str,
        avatar: Option<&'a Handle<Image>>,
        reference_points: i32,
        completed_games: u32,
        game_profiles: &'a PlayerGameProfiles,
        selected_game: ProfileGameTab,
        assets: &'a UiAssets,
    ) -> Self {
        Self {
            player_name,
            avatar,
            reference_points,
            completed_games,
            game_profiles,
            selected_game,
            assets,
        }
    }

    pub(crate) fn render(self, commands: &mut Commands, root: Entity, progress: f32) {
        let Self {
            player_name,
            avatar,
            reference_points,
            completed_games,
            game_profiles,
            selected_game,
            assets,
        } = self;
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
            GlobalZIndex(2000),
            FocusPolicy::Block,
            CozyModalBackdrop(CozyModalKind::Profile),
        ));

        let modal = add_cozy_panel(
            commands,
            overlay,
            Node {
                width: px(820),
                max_width: percent(90),
                min_height: px(520),
                padding: UiRect::all(px(24)),
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            },
            assets,
        );
        commands.entity(modal).insert((
            CozyModalPanel(CozyModalKind::Profile),
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
        add_text(commands, heading, "个人资料", 28.0, TEXT, assets);
        add_cozy_close_button(
            commands,
            heading,
            UiAction::Navigation(NavigationUiAction::ToggleProfile),
            assets,
        );
        spawn_node(
            commands,
            modal,
            Node {
                width: px(96),
                height: px(2),
                ..default()
            },
            Some(Color::srgb(0.64, 0.59, 0.93)),
        );
        ProfileIdentity::new(
            player_name,
            avatar,
            reference_points,
            completed_games,
            game_profiles.interactions.as_ref(),
            assets,
        )
        .render(commands, modal);
        spawn_node(
            commands,
            modal,
            Node {
                width: percent(100),
                height: px(1),
                ..default()
            },
            Some(Color::srgba(0.70, 0.68, 0.78, 0.36)),
        );
        ProfileArchive::new(game_profiles, selected_game, assets).render(commands, modal);
    }
}

struct ProfileIdentity<'a> {
    player_name: &'a str,
    avatar: Option<&'a Handle<Image>>,
    reference_points: i32,
    completed_games: u32,
    interactions: Option<&'a PlayerInteractionStats>,
    assets: &'a UiAssets,
}

impl<'a> ProfileIdentity<'a> {
    fn new(
        player_name: &'a str,
        avatar: Option<&'a Handle<Image>>,
        reference_points: i32,
        completed_games: u32,
        interactions: Option<&'a PlayerInteractionStats>,
        assets: &'a UiAssets,
    ) -> Self {
        Self {
            player_name,
            avatar,
            reference_points,
            completed_games,
            interactions,
            assets,
        }
    }

    fn render(self, commands: &mut Commands, parent: Entity) {
        let identity = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                min_height: px(118),
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
        add_text(
            commands,
            identity_text,
            self.player_name,
            27.0,
            TEXT,
            self.assets,
        );
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

    fn add_interaction_totals(&self, commands: &mut Commands, parent: Entity) {
        let stats = self.interactions.cloned().unwrap_or_default();
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
                    FocusPolicy::Pass,
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
                FocusPolicy::Pass,
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

struct ProfileArchive<'a> {
    game_profiles: &'a PlayerGameProfiles,
    selected_game: ProfileGameTab,
    assets: &'a UiAssets,
}

impl<'a> ProfileArchive<'a> {
    fn new(
        game_profiles: &'a PlayerGameProfiles,
        selected_game: ProfileGameTab,
        assets: &'a UiAssets,
    ) -> Self {
        Self {
            game_profiles,
            selected_game,
            assets,
        }
    }

    fn render(self, commands: &mut Commands, parent: Entity) {
        add_text(commands, parent, "游戏档案", 20.0, TEXT, self.assets);
        let archive_body = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            None,
        );
        let tabs = spawn_node(
            commands,
            archive_body,
            Node {
                width: percent(100),
                height: px(44),
                flex_direction: FlexDirection::Row,
                column_gap: px(6),
                ..default()
            },
            None,
        );
        for (game, label) in ProfileGameTab::ALL {
            self.add_tab(commands, tabs, game, label);
        }

        let content = spawn_node(
            commands,
            archive_body,
            Node {
                width: percent(100),
                min_height: px(166),
                flex_grow: 1.0,
                padding: UiRect::all(px(16)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                column_gap: px(16),
                ..default()
            },
            Some(Color::srgb(0.30, 0.31, 0.34)),
        );
        commands.entity(content).insert(ProfileGameContent);
        let rows = match self.selected_game {
            ProfileGameTab::QiGui523 => qigui523_profile_rows(self.game_profiles.qigui523.as_ref()),
            ProfileGameTab::TexasHoldem => {
                texas_holdem_profile_rows(self.game_profiles.texas_holdem.as_ref())
            }
            ProfileGameTab::Shengji => shengji_profile_rows(self.game_profiles.shengji.as_ref()),
            ProfileGameTab::Uno => uno_profile_rows(self.game_profiles.uno.as_ref()),
            ProfileGameTab::Mahjong => mahjong_profile_rows(self.game_profiles.mahjong.as_ref()),
        };
        let rows_per_column = rows.len().div_ceil(4).max(1);
        for column_index in 0..4 {
            let column = spawn_node(
                commands,
                content,
                Node {
                    min_width: px(0),
                    flex_basis: px(0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(7),
                    ..default()
                },
                None,
            );
            commands.entity(column).insert(ProfileGameColumn);
            for (label, value) in rows
                .iter()
                .skip(column_index * rows_per_column)
                .take(rows_per_column)
            {
                self.add_row(commands, column, label, value);
            }
        }
    }

    fn add_row(&self, commands: &mut Commands, parent: Entity, label: &str, value: &str) {
        let row = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                min_height: px(23),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: px(8),
                ..default()
            },
            None,
        );
        add_text(commands, row, label, 13.0, MUTED, self.assets);
        add_text(commands, row, value, 14.0, TEXT, self.assets);
    }

    fn add_tab(&self, commands: &mut Commands, parent: Entity, game: ProfileGameTab, label: &str) {
        let selected = game == self.selected_game;
        let button = commands
            .spawn((
                Button,
                UiAction::Navigation(NavigationUiAction::SelectProfileGameTab(game)),
                Node {
                    min_width: px(0),
                    height: percent(100),
                    flex_basis: px(0),
                    flex_grow: 1.0,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(if selected {
                    Color::srgb(0.30, 0.31, 0.34)
                } else {
                    TAB_IDLE
                }),
                ProfileGameTabButton,
            ))
            .id();
        if selected {
            commands.entity(button).insert(SelectedProfileGameTab);
        }
        commands.entity(parent).add_child(button);
        add_text(
            commands,
            button,
            label,
            15.0,
            if selected {
                Color::srgb(0.85, 0.82, 1.0)
            } else {
                TEXT
            },
            self.assets,
        );
    }
}
