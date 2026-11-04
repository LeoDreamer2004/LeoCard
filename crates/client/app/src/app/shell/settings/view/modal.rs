//! 全局设置窗口与页签组合。

use super::super::{SelectedSettingsTab, SettingsTabButton};
use super::about::AboutSettings;
use super::appearance::TableAppearanceSettings;
use super::sound::render_sound_settings;
use crate::app::presentation::{
    PAGE_FRAME_TITLE, TEXT, add_text, add_title, card_background_image, page_frame_skin,
};
use crate::app::runtime::{AppearancePreferences, UiAssets};
use crate::app::shell::{
    CozyModalBackdrop, CozyModalKind, CozyModalPanel, NavigationUiAction, SettingsTab,
    SettingsUiAction, UiAction, UpdateManager, add_configuration_scroll, add_cozy_close_button,
    cozy_backdrop_color, cozy_panel_transform,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_ui::{PageFrame, UiComposer, decorate_panel, sliced_image, spawn_node};

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
        commands: &mut UiComposer,
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
            Pickable::default(),
            CozyModalBackdrop(CozyModalKind::Settings),
        ));
        let mut frame = PageFrame::new(page_frame_skin(assets));
        frame.node.width = px(680);
        frame.node.height = px(520);
        frame.node.max_width = percent(92);
        frame.node.max_height = percent(92);
        frame.heading.height = px(48);
        let frame = frame.mount(commands, overlay);
        commands.entity(frame.panel).insert((
            CozyModalPanel(CozyModalKind::Settings),
            cozy_panel_transform(progress),
        ));
        let heading = frame.heading;
        add_title(
            commands,
            heading,
            "游戏设置",
            28.0,
            PAGE_FRAME_TITLE,
            assets,
        );
        add_cozy_close_button(
            commands,
            heading,
            UiAction::Navigation(NavigationUiAction::ToggleSettings),
            assets,
        );
        let tabbed = spawn_node(
            commands,
            frame.content,
            Node {
                width: percent(100),
                min_height: px(0),
                flex_grow: 1.0,
                flex_basis: px(0),
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
        let content = spawn_node(
            commands,
            tabbed,
            Node {
                min_width: px(0),
                min_height: px(0),
                flex_basis: px(0),
                flex_grow: 1.0,
                padding: UiRect::all(px(28)),
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            },
            None,
        );
        decorate_panel(
            commands,
            content,
            sliced_image(assets.home.settings_page.clone(), 20.0, 1.0)
                .with_color(Color::srgb(0.55, 0.54, 0.63)),
        );
        // About owns a full-height column so its update controls remain at the bottom.
        // Configuration lists scroll only when necessary and do not draw a scrollbar.
        let content = if selected_tab == SettingsTab::About {
            content
        } else {
            add_configuration_scroll(
                commands,
                content,
                format!("settings:{selected_tab:?}"),
                12.0,
            )
        };
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
    commands: &mut UiComposer,
    parent: Entity,
    tab: SettingsTab,
    label: &str,
    selected_tab: SettingsTab,
    assets: &UiAssets,
) {
    let selected = tab == selected_tab;
    let tab_image = card_background_image(
        if selected {
            assets.home.game_card_hover.clone()
        } else {
            assets.home.game_card.clone()
        },
        1.0,
    );
    let button = commands.child(
        parent,
        (
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
        ),
    );
    if selected {
        commands.entity(button).insert(SelectedSettingsTab);
    }
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
