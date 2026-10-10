//! Shared settings slider for brightness, vignette and volume.

use super::super::super::{table_appearance_fraction, table_appearance_label};
use super::super::CozySettingsSlider;
use crate::app::presentation::{
    TEXT, TableAppearanceIndicator, TableAppearanceLabel, TableAppearanceSetting,
    TableAppearanceSlider, add_text, spawn_node,
};
use crate::app::runtime::{AppearancePreferences, UiAssets};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::{RelativeCursorPosition, VisualBox};
use bevy::ui_widgets::Button;

pub(super) struct SettingsSlider<'a> {
    form: &'a AppearancePreferences,
    assets: &'a UiAssets,
}

impl<'a> SettingsSlider<'a> {
    pub(super) fn new(form: &'a AppearancePreferences, assets: &'a UiAssets) -> Self {
        Self { form, assets }
    }

    pub(super) fn render(
        &self,
        commands: &mut Commands,
        parent: Entity,
        setting: TableAppearanceSetting,
    ) {
        let form = self.form;
        let assets = self.assets;
        let fraction = table_appearance_fraction(setting, form);
        let group = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                ..default()
            },
            None,
        );
        let label = add_text(
            commands,
            group,
            table_appearance_label(setting, form),
            15.0,
            TEXT,
            assets,
        );
        commands.entity(label).insert(TableAppearanceLabel(setting));
        let slider = commands
            .spawn((
                Button,
                TableAppearanceSlider(setting),
                RelativeCursorPosition::default(),
                Node {
                    width: percent(100),
                    height: px(38),
                    position_type: PositionType::Relative,
                    ..default()
                },
            ))
            .id();
        commands.entity(group).add_child(slider);
        let mut track_image = ImageNode::new(assets.home.slider.clone()).with_mode(
            NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(32.0),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 0.375,
            }),
        );
        track_image.visual_box = VisualBox::BorderBox;
        let track = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(7),
                    height: px(24),
                    ..default()
                },
                track_image,
                Pickable::IGNORE,
            ))
            .id();
        commands.entity(slider).add_child(track);
        let knob = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(fraction * 100.0),
                    top: px(2),
                    width: px(14),
                    height: px(34),
                    ..default()
                },
                ImageNode::new(assets.home.slider_handle.clone()).with_mode(NodeImageMode::Stretch),
            ))
            .id();
        commands.entity(slider).add_child(knob);
        commands.entity(knob).insert((
            TableAppearanceIndicator(setting),
            CozySettingsSlider {
                owner: slider,
                track,
                hover: 0.0,
            },
            UiTransform::from_translation(Val2::px(-7.0, 0.0)),
            Pickable::IGNORE,
        ));
    }
}
