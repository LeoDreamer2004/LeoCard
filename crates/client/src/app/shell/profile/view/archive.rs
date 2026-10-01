use super::super::super::UiAction;
use crate::app::shell::ProfileUiAction;

use super::super::{
    ProfileGameColumn, ProfileGameContent, ProfileGameTab, ProfileGameTabButton,
    SelectedProfileGameTab, TAB_IDLE, mahjong_profile_rows, qigui523_profile_rows,
    shengji_profile_rows, texas_holdem_profile_rows, uno_profile_rows,
};
use crate::app::presentation::{MUTED, TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use leocard_protocol::PlayerGameProfiles;

pub(super) struct ProfileArchive<'a> {
    game_profiles: &'a PlayerGameProfiles,
    selected_game: ProfileGameTab,
    assets: &'a UiAssets,
}

impl<'a> ProfileArchive<'a> {
    pub(super) fn new(
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

    pub(super) fn render(self, commands: &mut Commands, parent: Entity) {
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
                UiAction::Profile(ProfileUiAction::SelectGameTab(game)),
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
