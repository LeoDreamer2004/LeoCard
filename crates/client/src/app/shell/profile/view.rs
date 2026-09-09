//! 玩家档案弹窗、游戏标签和互动统计视图。

use super::super::{NavigationUiAction, UiAction};
use super::{
    ProfileGameColumn, ProfileGameContent, ProfileGameTab, ProfileGameTabButton, ProfileStat,
    SelectedProfileGameTab, qigui523_profile_rows, reference_level, shengji_profile_rows,
    texas_holdem_profile_rows, uno_profile_rows,
};
use crate::app::presentation::{
    ACCENT, BORDER, ButtonKind, ButtonTint, HEADER_BG, MUTED, PANEL, PanelSkin, TEXT,
    add_action_button, add_avatar, add_panel, add_section_title, add_text, spawn_node,
};
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

    pub(crate) fn render(self, commands: &mut Commands, root: Entity) {
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
            Some(Color::srgba(0.005, 0.015, 0.012, 0.76)),
        );
        commands
            .entity(overlay)
            .insert((GlobalZIndex(2000), FocusPolicy::Block));

        let modal = add_panel(
            commands,
            overlay,
            Node {
                width: px(820),
                max_width: percent(90),
                min_height: px(520),
                flex_direction: FlexDirection::Column,
                row_gap: px(14),
                ..default()
            },
            PANEL,
            PanelSkin::Window,
            assets,
        );
        add_section_title(commands, modal, "个人资料", assets);
        ProfileIdentity::new(
            player_name,
            avatar,
            reference_points,
            completed_games,
            game_profiles.interactions.as_ref(),
            assets,
        )
        .render(commands, modal);
        ProfileArchive::new(game_profiles, selected_game, assets).render(commands, modal);

        let actions = spawn_node(
            commands,
            modal,
            Node {
                width: percent(100),
                align_items: AlignItems::FlexEnd,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            None,
        );
        add_action_button(
            commands,
            actions,
            "关闭",
            UiAction::Navigation(NavigationUiAction::ToggleProfile),
            ButtonKind::Secondary,
            assets,
        );
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
                min_height: px(138),
                padding: UiRect::all(px(16)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(12),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(8)),
                ..default()
            },
            Some(HEADER_BG.with_alpha(0.86)),
        );
        commands.entity(identity).insert(BorderColor::all(BORDER));

        let avatar_area = spawn_node(
            commands,
            identity,
            Node {
                width: px(96),
                min_width: px(96),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(8),
                ..default()
            },
            None,
        );
        add_avatar(
            commands,
            avatar_area,
            self.player_name,
            self.avatar,
            82.0,
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
            25.0,
            TEXT,
            self.assets,
        );
        self.add_interaction_totals(commands, identity_text);

        let stats = spawn_node(
            commands,
            identity,
            Node {
                width: px(300),
                min_width: px(300),
                flex_direction: FlexDirection::Row,
                column_gap: px(8),
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
        self.add_stat(
            commands,
            stats,
            "等级",
            reference_level(self.reference_points),
        );
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
            add_text(commands, item, count.to_string(), 16.0, ACCENT, self.assets);
        }
    }

    fn add_stat(
        &self,
        commands: &mut Commands,
        parent: Entity,
        label: &str,
        value: impl Into<String>,
    ) {
        let card = spawn_node(
            commands,
            parent,
            Node {
                min_width: px(0),
                min_height: px(76),
                flex_basis: px(0),
                flex_grow: 1.0,
                padding: UiRect::axes(px(4), px(10)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(5),
                border: UiRect::ZERO,
                ..default()
            },
            None,
        );
        commands.entity(card).insert(ProfileStat);
        add_text(commands, card, label, 12.0, MUTED, self.assets);
        add_text(commands, card, value, 21.0, ACCENT, self.assets);
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
        add_text(commands, parent, "游戏档案", 16.0, TEXT, self.assets);
        let tabs = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                height: px(42),
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
            parent,
            Node {
                width: percent(100),
                min_height: px(166),
                flex_grow: 1.0,
                padding: UiRect::all(px(14)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                column_gap: px(12),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(8)),
                ..default()
            },
            Some(HEADER_BG.with_alpha(0.54)),
        );
        commands
            .entity(content)
            .insert((ProfileGameContent, BorderColor::all(BORDER)));
        let rows = match self.selected_game {
            ProfileGameTab::QiGui523 => qigui523_profile_rows(self.game_profiles.qigui523.as_ref()),
            ProfileGameTab::TexasHoldem => {
                texas_holdem_profile_rows(self.game_profiles.texas_holdem.as_ref())
            }
            ProfileGameTab::Shengji => shengji_profile_rows(self.game_profiles.shengji.as_ref()),
            ProfileGameTab::Uno => uno_profile_rows(self.game_profiles.uno.as_ref()),
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
                    row_gap: px(5),
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
                min_height: px(22),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: px(8),
                ..default()
            },
            None,
        );
        add_text(commands, row, label, 12.5, MUTED, self.assets);
        add_text(commands, row, value, 13.0, TEXT, self.assets);
    }

    fn add_tab(&self, commands: &mut Commands, parent: Entity, game: ProfileGameTab, label: &str) {
        let selected = game == self.selected_game;
        let normal = if selected {
            Color::srgb(0.36, 0.48, 0.32)
        } else {
            Color::srgb(0.22, 0.32, 0.29)
        };
        let button = commands
            .spawn((
                Button,
                UiAction::Navigation(NavigationUiAction::SelectProfileGameTab(game)),
                ButtonTint {
                    normal,
                    hovered: if selected {
                        Color::srgb(0.43, 0.55, 0.36)
                    } else {
                        Color::srgb(0.30, 0.42, 0.36)
                    },
                    pressed: Color::srgb(0.18, 0.28, 0.24),
                },
                Node {
                    min_width: px(0),
                    height: percent(100),
                    flex_basis: px(0),
                    flex_grow: 1.0,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::bottom(px(if selected { 3 } else { 1 })),
                    border_radius: BorderRadius::top(px(7)),
                    ..default()
                },
                ImageNode::new(self.assets.controls.secondary_button.clone())
                    .with_mode(NodeImageMode::Stretch)
                    .with_color(normal),
                BorderColor::all(if selected { ACCENT } else { BORDER }),
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
            14.0,
            if selected { Color::WHITE } else { MUTED },
            self.assets,
        );
    }
}
