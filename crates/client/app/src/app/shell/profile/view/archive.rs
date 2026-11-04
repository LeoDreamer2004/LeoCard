use super::super::super::UiAction;
use super::super::scroll::{ArchiveEdgeSide, ProfileArchiveEdge};
use super::super::{
    ProfileGameContent, ProfileGameTab, ProfileGameTabButton, SelectedProfileGameTab, TAB_IDLE,
    blackjack_profile_rows, doudizhu_profile_rows, mahjong_profile_rows, mc_guess_profile_rows,
    qigui523_profile_rows, shengji_profile_rows, texas_profile_rows, uno_profile_rows,
};
use crate::app::presentation::{MUTED, TEXT, add_text, add_title};
use crate::app::runtime::UiAssets;
use crate::app::shell::ProfileUiAction;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::{BackgroundGradient, ColorStop, LinearGradient, RelativeCursorPosition};
use bevy::ui_widgets::Button;
use leocard_protocol::PlayerGameProfiles;
use leocard_ui::{TouchDragTarget, UiComposer, spawn_node};

const TAB_HEIGHT: f32 = 44.0;
const CONTENT_PADDING: f32 = 16.0;
const ROW_HEIGHT: f32 = 24.0;
const ROW_GAP: f32 = 8.0;
const ROWS_PER_COLUMN: u16 = 6;
const CONTENT_HEIGHT: f32 =
    2.0 * CONTENT_PADDING + ROWS_PER_COLUMN as f32 * (ROW_HEIGHT + ROW_GAP) - ROW_GAP;

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

    pub(super) fn render(self, commands: &mut UiComposer, parent: Entity) {
        add_title(commands, parent, "游戏档案", 20.0, TEXT, self.assets);
        let archive_body = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                height: px(TAB_HEIGHT + CONTENT_HEIGHT),
                min_height: px(TAB_HEIGHT + CONTENT_HEIGHT),
                flex_shrink: 0.0,
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
                height: px(TAB_HEIGHT),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Row,
                column_gap: px(6),
                ..default()
            },
            None,
        );
        for (game, label) in ProfileGameTab::ALL {
            self.add_tab(commands, tabs, game, label);
        }

        let background = Color::srgb(0.30, 0.31, 0.34);
        let viewport = spawn_node(
            commands,
            archive_body,
            Node {
                width: percent(100),
                height: px(CONTENT_HEIGHT),
                min_height: px(CONTENT_HEIGHT),
                position_type: PositionType::Relative,
                ..default()
            },
            Some(background),
        );
        let content = commands.keyed_child(
            viewport,
            format!("stats:{:?}", self.selected_game),
            (
                Node {
                    width: percent(100),
                    height: percent(100),
                    display: Display::Grid,
                    padding: UiRect::all(px(CONTENT_PADDING)),
                    grid_auto_flow: GridAutoFlow::Column,
                    grid_template_rows: RepeatedGridTrack::px(ROWS_PER_COLUMN, ROW_HEIGHT),
                    grid_auto_columns: vec![GridTrack::px(220.0)],
                    row_gap: px(ROW_GAP),
                    column_gap: px(20),
                    align_content: AlignContent::Start,
                    overflow: Overflow::scroll_x(),
                    ..default()
                },
                ProfileGameContent,
                TouchDragTarget,
                RelativeCursorPosition::default(),
            ),
        );
        commands.entity(content).init(ScrollPosition::default());
        for side in [ArchiveEdgeSide::Left, ArchiveEdgeSide::Right] {
            let mut node = Node {
                position_type: PositionType::Absolute,
                top: px(0),
                bottom: px(0),
                width: px(40),
                ..default()
            };
            let stops = vec![
                ColorStop::percent(background.with_alpha(0.0), 0.0),
                ColorStop::percent(background, 100.0),
            ];
            let gradient = match side {
                ArchiveEdgeSide::Left => {
                    node.left = px(0);
                    LinearGradient::to_left(stops)
                }
                ArchiveEdgeSide::Right => {
                    node.right = px(0);
                    LinearGradient::to_right(stops)
                }
            };
            commands.child(
                viewport,
                (
                    node,
                    BackgroundGradient::from(gradient),
                    ProfileArchiveEdge {
                        viewport: content,
                        side,
                    },
                    Visibility::Hidden,
                    ZIndex(2),
                    Pickable::IGNORE,
                ),
            );
        }
        let rows = match self.selected_game {
            ProfileGameTab::Blackjack => {
                blackjack_profile_rows(self.game_profiles.blackjack.as_ref())
            }
            ProfileGameTab::Doudizhu => doudizhu_profile_rows(self.game_profiles.doudizhu.as_ref()),
            ProfileGameTab::QiGui523 => qigui523_profile_rows(self.game_profiles.qigui523.as_ref()),
            ProfileGameTab::Texas => texas_profile_rows(self.game_profiles.texas.as_ref()),
            ProfileGameTab::Shengji => shengji_profile_rows(self.game_profiles.shengji.as_ref()),
            ProfileGameTab::Uno => uno_profile_rows(self.game_profiles.uno.as_ref()),
            ProfileGameTab::Mahjong => mahjong_profile_rows(self.game_profiles.mahjong.as_ref()),
            ProfileGameTab::McGuess => mc_guess_profile_rows(self.game_profiles.mc_guess.as_ref()),
        };
        for (label, value) in rows {
            self.add_row(commands, content, label, &value);
        }
    }

    fn add_row(&self, commands: &mut UiComposer, parent: Entity, label: &str, value: &str) {
        let row = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                height: px(ROW_HEIGHT),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: px(8),
                ..default()
            },
            None,
        );
        for (text, size, color) in [(label, 13.0, MUTED), (value, 14.0, TEXT)] {
            let label = add_text(commands, row, text, size, color, self.assets);
            commands.entity(label).insert(TextLayout::no_wrap());
        }
    }

    fn add_tab(
        &self,
        commands: &mut UiComposer,
        parent: Entity,
        game: ProfileGameTab,
        label: &str,
    ) {
        let selected = game == self.selected_game;
        let button = commands.child(
            parent,
            (
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
            ),
        );
        if selected {
            commands.entity(button).insert(SelectedProfileGameTab);
        }
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
