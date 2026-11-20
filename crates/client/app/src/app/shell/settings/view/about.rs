//! Product information and update actions.

#[cfg(not(feature = "web"))]
use super::super::super::{
    CozyButtonVariant, UpdateManager, UpdateState, add_cozy_button_variant, settings_update_label,
};
use crate::app::presentation::{MUTED, TEXT, add_text, add_title};
use crate::app::runtime::UiAssets;
#[cfg(feature = "web")]
use crate::app::shell::NavigationUiAction;
#[cfg(not(feature = "web"))]
use crate::app::shell::UpdateUiAction;
use crate::app::shell::{UiAction, add_cozy_button_with_icon};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

pub(super) struct AboutSettings<'a> {
    #[cfg(not(feature = "web"))]
    updater: &'a UpdateManager,
    assets: &'a UiAssets,
}

impl<'a> AboutSettings<'a> {
    pub(super) fn new(
        #[cfg(not(feature = "web"))] updater: &'a UpdateManager,
        assets: &'a UiAssets,
    ) -> Self {
        Self {
            #[cfg(not(feature = "web"))]
            updater,
            assets,
        }
    }

    pub(super) fn render(self, commands: &mut UiComposer, parent: Entity) {
        let identity = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                min_height: px(0),
                flex_grow: 1.0,
                flex_basis: px(0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(8),
                ..default()
            },
            None,
        );
        commands.child(
            identity,
            (
                Node {
                    position_type: PositionType::Absolute,
                    right: px(0),
                    bottom: px(0),
                    height: percent(90),
                    max_height: px(224),
                    aspect_ratio: Some(1.0),
                    ..default()
                },
                ImageNode::new(self.assets.home.about_mascot.clone())
                    .with_color(Color::WHITE.with_alpha(0.12)),
                ZIndex(-1),
                Pickable::IGNORE,
            ),
        );
        add_title(
            commands,
            identity,
            "LeoCard",
            34.0,
            Color::srgb(0.85, 0.82, 1.0),
            self.assets,
        );
        add_text(
            commands,
            identity,
            "2216 特供游戏桌",
            16.0,
            TEXT,
            self.assets,
        );
        add_text(
            commands,
            identity,
            format!("版本 v{}", env!("CARGO_PKG_VERSION")),
            14.0,
            MUTED,
            self.assets,
        );
        #[cfg(feature = "web")]
        add_cozy_button_with_icon(
            commands,
            identity,
            "GitHub",
            UiAction::Navigation(NavigationUiAction::OpenProject),
            self.assets,
            px(124),
            40.0,
            Some(self.assets.controls.github_mark.clone()),
        );
        #[cfg(not(feature = "web"))]
        SoftwareUpdateSettings::new(self.updater, self.assets).render(commands, parent);
    }
}

#[cfg(not(feature = "web"))]
struct SoftwareUpdateSettings<'a> {
    #[cfg(not(feature = "web"))]
    updater: &'a UpdateManager,
    assets: &'a UiAssets,
}

#[cfg(not(feature = "web"))]
impl<'a> SoftwareUpdateSettings<'a> {
    fn new(updater: &'a UpdateManager, assets: &'a UiAssets) -> Self {
        Self {
            #[cfg(not(feature = "web"))]
            updater,
            assets,
        }
    }

    fn render(self, commands: &mut UiComposer, parent: Entity) {
        let updater = self.updater;
        let assets = self.assets;
        spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                height: px(1),
                flex_shrink: 0.0,
                margin: UiRect::vertical(px(4)),
                ..default()
            },
            Some(Color::srgba(0.70, 0.68, 0.78, 0.36)),
        );
        let update_row = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                min_height: px(48),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: px(12),
                ..default()
            },
            None,
        );
        let update_text = spawn_node(
            commands,
            update_row,
            Node {
                min_width: px(0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                row_gap: px(3),
                ..default()
            },
            None,
        );
        add_text(commands, update_text, "软件更新", 20.0, TEXT, assets);
        let star = add_text(
            commands,
            update_text,
            "可以给一个小小的Star吗? 谢谢辽~",
            13.0,
            MUTED,
            assets,
        );
        commands
            .entity(star)
            .insert(TextLayout::default().with_no_wrap());
        let update_actions = spawn_node(
            commands,
            update_row,
            Node {
                flex_direction: FlexDirection::Row,
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                column_gap: px(10),
                ..default()
            },
            None,
        );
        add_cozy_button_with_icon(
            commands,
            update_actions,
            "GitHub",
            UiAction::Update(UpdateUiAction::OpenGitHubRepository),
            assets,
            px(124),
            40.0,
            Some(assets.controls.github_mark.clone()),
        );
        add_cozy_button_variant(
            commands,
            update_actions,
            settings_update_label(&updater.state),
            match updater.state {
                UpdateState::Ready { .. } => UiAction::Update(UpdateUiAction::RestartToUpdate),
                _ => UiAction::Update(UpdateUiAction::StartUpdate),
            },
            assets,
            px(124),
            40.0,
            CozyButtonVariant::Cool,
        );
    }
}
