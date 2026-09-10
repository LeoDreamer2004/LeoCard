//! 桌面外观滑块输入。

use crate::app::presentation::{
    MAX_TABLE_BRIGHTNESS, MAX_TABLE_VIGNETTE, MIN_TABLE_BRIGHTNESS, MIN_TABLE_VIGNETTE,
    TableAppearanceIndicator, TableAppearanceIndicatorPart, TableAppearanceLabel,
    TableAppearanceSetting, TableAppearanceSlider, TableBackground, TableBackgroundMaterial,
};
use crate::app::runtime::{
    AppearancePreferences, normalize_table_brightness, save_appearance_preferences,
};
use bevy::audio::Volume;
use bevy::log::warn;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

pub(crate) fn table_brightness_fraction(brightness: f32) -> f32 {
    ((normalize_table_brightness(brightness) - MIN_TABLE_BRIGHTNESS)
        / (MAX_TABLE_BRIGHTNESS - MIN_TABLE_BRIGHTNESS))
        .clamp(0.0, 1.0)
}

pub(crate) fn slider_fraction_from_relative_x(relative_x: f32) -> f32 {
    (relative_x + 0.5).clamp(0.0, 1.0)
}

pub(crate) fn table_appearance_fraction(
    setting: TableAppearanceSetting,
    form: &AppearancePreferences,
) -> f32 {
    match setting {
        TableAppearanceSetting::Brightness => table_brightness_fraction(form.table_brightness),
        TableAppearanceSetting::Vignette => ((form.table_vignette - MIN_TABLE_VIGNETTE)
            / (MAX_TABLE_VIGNETTE - MIN_TABLE_VIGNETTE))
            .clamp(0.0, 1.0),
        TableAppearanceSetting::Volume => form.audio_volume,
    }
}

pub(crate) fn table_appearance_label(
    setting: TableAppearanceSetting,
    form: &AppearancePreferences,
) -> String {
    match setting {
        TableAppearanceSetting::Brightness => {
            format!("亮度  {:.0}%", form.table_brightness * 100.0)
        }
        TableAppearanceSetting::Vignette => {
            format!("四周视角阴影  {:.0}%", form.table_vignette * 100.0)
        }
        TableAppearanceSetting::Volume => format!("音量  {:.0}%", form.audio_volume * 100.0),
    }
}

fn set_table_appearance_from_fraction(
    setting: TableAppearanceSetting,
    fraction: f32,
    form: &mut AppearancePreferences,
) {
    match setting {
        TableAppearanceSetting::Brightness => {
            form.table_brightness =
                MIN_TABLE_BRIGHTNESS + fraction * (MAX_TABLE_BRIGHTNESS - MIN_TABLE_BRIGHTNESS);
        }
        TableAppearanceSetting::Vignette => {
            form.table_vignette =
                MIN_TABLE_VIGNETTE + fraction * (MAX_TABLE_VIGNETTE - MIN_TABLE_VIGNETTE);
        }
        TableAppearanceSetting::Volume => form.audio_volume = fraction,
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects independent pointer state and appearance resources"
)]
pub(crate) fn handle_table_appearance_sliders(
    mouse: Res<ButtonInput<MouseButton>>,
    sliders: Query<(
        &Interaction,
        &RelativeCursorPosition,
        &TableAppearanceSlider,
    )>,
    mut indicators: Query<(&TableAppearanceIndicator, &mut Node)>,
    mut labels: Query<(&TableAppearanceLabel, &mut Text)>,
    backgrounds: Query<&MaterialNode<TableBackgroundMaterial>, With<TableBackground>>,
    mut materials: ResMut<Assets<TableBackgroundMaterial>>,
    mut global_volume: ResMut<GlobalVolume>,
    mut form: ResMut<AppearancePreferences>,
    mut dragging: Local<Option<TableAppearanceSetting>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        *dragging = sliders.iter().find_map(|(interaction, _, slider)| {
            (*interaction == Interaction::Pressed).then_some(slider.0)
        });
    }

    if let Some(setting) = *dragging
        && mouse.pressed(MouseButton::Left)
        && let Some(position) = sliders
            .iter()
            .find(|(_, _, slider)| slider.0 == setting)
            .and_then(|(_, cursor, _)| cursor.normalized.map(|position| position.x))
    {
        let fraction = slider_fraction_from_relative_x(position);
        let before = table_appearance_fraction(setting, &form);
        if (before - fraction).abs() > 0.001 {
            set_table_appearance_from_fraction(setting, fraction, &mut form);
            for (indicator, mut node) in &mut indicators {
                if indicator.setting != setting {
                    continue;
                }
                match indicator.part {
                    TableAppearanceIndicatorPart::Fill => node.width = percent(fraction * 100.0),
                    TableAppearanceIndicatorPart::Knob => node.left = percent(fraction * 100.0),
                }
            }
            for (label, mut text) in &mut labels {
                if label.0 == setting {
                    text.0 = table_appearance_label(setting, &form);
                }
            }
            for background in &backgrounds {
                if let Some(mut material) = materials.get_mut(background) {
                    material.params.x = form.table_vignette;
                    material.params.y = form.table_brightness;
                }
            }
            global_volume.volume = Volume::Linear(form.audio_volume);
        }
    }

    if dragging.is_some() && mouse.just_released(MouseButton::Left) {
        *dragging = None;
        if let Err(error) = save_appearance_preferences(&form) {
            warn!("{error}");
        }
    }
}
