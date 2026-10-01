//! 全局设置窗口与页签组合。

use crate::app::shell::SettingsUiAction;

use crate::app::shell::{cozy_backdrop_color, cozy_panel_transform};

use super::super::super::{
    CozyModalBackdrop, CozyModalKind, CozyModalPanel, NavigationUiAction, SettingsTab, UiAction,
    UpdateManager, add_cozy_close_button, add_cozy_panel,
};
use super::super::{SelectedSettingsTab, SettingsTabButton};
use super::about::AboutSettings;
use super::appearance::TableAppearanceSettings;
use super::sound::render_sound_settings;
use crate::app::presentation::{TEXT, add_text, spawn_node};
use crate::app::runtime::{AppearancePreferences, UiAssets};
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, VisualBox};

pub(crate) struct SettingsModal<'a> {
    form: &'a AppearancePreferences,
    updater: &'a UpdateManager,
    assets: &'a UiAssets,
}

impl<'a> SettingsModal<'a> {
    pub(crate) fn new(
        form: &'a AppearancePreferences,
        updater: &'a UpdateManager,
        assets: &'a UiAssets,
    ) -> Self {
        Self {
            form,
            updater,
            assets,
        }
    }

    pub(crate) fn render(
        &self,
        commands: &mut Commands,
        root: Entity,
        progress: f32,
        selected_tab: SettingsTab,
    ) {
        let form = self.form;
        let updater = self.updater;
        let assets = self.assets;
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
            CozyModalBackdrop(CozyModalKind::Settings),
        ));
        let modal = add_cozy_panel(
            commands,
            overlay,
            Node {
                width: px(680),
                max_width: percent(92),
                padding: UiRect::all(px(24)),
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            },
            assets,
        );
        commands.entity(modal).insert((
            CozyModalPanel(CozyModalKind::Settings),
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
        add_text(commands, heading, "游戏设置", 28.0, TEXT, assets);
        add_cozy_close_button(
            commands,
            heading,
            UiAction::Navigation(NavigationUiAction::ToggleSettings),
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
        let tabbed = spawn_node(
            commands,
            modal,
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Row,
                column_gap: px(14),
                ..default()
            },
            None,
        );
        let tabs = spawn_node(
            commands,
            tabbed,
            Node {
                width: px(100),
                min_width: px(100),
                padding: UiRect::top(px(18)),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..default()
            },
            None,
        );
        for (tab, label) in [
            (SettingsTab::Appearance, "外观"),
            (SettingsTab::Sound, "声音"),
            (SettingsTab::About, "关于"),
        ] {
            add_settings_tab(commands, tabs, tab, label, selected_tab, assets);
        }
        let mut content_image = ImageNode::new(assets.home.settings_page.clone()).with_mode(
            NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(20.0),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 1.0,
            }),
        );
        content_image.visual_box = VisualBox::BorderBox;
        content_image.color = Color::srgb(0.55, 0.54, 0.63);
        let content = commands
            .spawn((
                Node {
                    min_width: px(0),
                    min_height: px(390),
                    flex_grow: 1.0,
                    padding: UiRect::all(px(22)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(12),
                    ..default()
                },
                content_image,
            ))
            .id();
        commands.entity(tabbed).add_child(content);
        match selected_tab {
            SettingsTab::Appearance => {
                TableAppearanceSettings::new(form, assets).render(commands, content);
            }
            SettingsTab::Sound => {
                render_sound_settings(commands, content, form, assets);
            }
            SettingsTab::About => {
                AboutSettings::new(updater, assets).render(commands, content);
            }
        }
    }
}

fn add_settings_tab(
    commands: &mut Commands,
    parent: Entity,
    tab: SettingsTab,
    label: &str,
    selected_tab: SettingsTab,
    assets: &UiAssets,
) {
    let selected = tab == selected_tab;
    let mut tab_image = ImageNode::new(if selected {
        assets.home.game_card_hover.clone()
    } else {
        assets.home.game_card.clone()
    })
    .with_mode(NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect::all(16.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    }));
    tab_image.visual_box = VisualBox::BorderBox;
    let button = commands
        .spawn((
            Button,
            UiAction::Settings(SettingsUiAction::SelectTab(tab)),
            Node {
                width: percent(100),
                height: px(48),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            tab_image,
        ))
        .id();
    if selected {
        commands.entity(button).insert(SelectedSettingsTab);
    }
    commands.entity(parent).add_child(button);
    let text = add_text(
        commands,
        button,
        label,
        17.0,
        if selected {
            Color::srgb(0.85, 0.82, 1.0)
        } else {
            TEXT
        },
        assets,
    );
    commands
        .entity(button)
        .insert(SettingsTabButton { label: text });
}
