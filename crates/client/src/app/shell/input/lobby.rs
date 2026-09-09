//! 大厅座位、规则提示和桌面外观输入。

use crate::app::presentation::{
    ACCENT, LobbyEmptySeatLabel, LobbyEmptySeatRing, LobbySeatHover, LobbySeatVisual,
    MAX_TABLE_BRIGHTNESS, MAX_TABLE_VIGNETTE, MIN_TABLE_BRIGHTNESS, MIN_TABLE_VIGNETTE, MUTED,
    PANEL_ALT, RuleHelp, TableAppearanceIndicator, TableAppearanceIndicatorPart,
    TableAppearanceLabel, TableAppearanceSetting, TableAppearanceSlider, TableBackground,
    TableBackgroundMaterial,
};
use crate::app::runtime::{
    AppearancePreferences, ClientResource, normalize_table_brightness, save_appearance_preferences,
};
use bevy::audio::Volume;
use bevy::log::warn;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
#[cfg(feature = "developer")]
use leocard_protocol::{ClientCommand, SeatId};

pub(crate) fn animate_lobby_seat_hover(
    time: Res<Time>,
    mut seats: Query<(&Interaction, &mut LobbySeatHover)>,
    mut visuals: Query<(&LobbySeatVisual, &mut UiTransform)>,
    mut empty_rings: Query<(&LobbyEmptySeatRing, &mut BorderColor, &mut BackgroundColor)>,
    mut empty_labels: Query<(&LobbyEmptySeatLabel, &mut Text, &mut TextColor)>,
) {
    let response = 1.0 - (-time.delta_secs() * 16.0).exp();
    for (interaction, mut hover) in &mut seats {
        let hovered = !matches!(interaction, Interaction::None);
        let target = if hovered { 1.0 } else { 0.0 };
        hover.amount += (target - hover.amount) * response;
        if (hover.amount - target).abs() < 0.002 {
            hover.amount = target;
        }
        let direction = match hover.seat {
            0 => Vec2::new(0.2, -1.0),
            1 => Vec2::new(1.0, 0.0),
            2 => Vec2::new(0.2, 1.0),
            3 => Vec2::new(-0.2, 1.0),
            4 => Vec2::new(-1.0, 0.0),
            5 => Vec2::new(-0.2, -1.0),
            _ => Vec2::ZERO,
        }
        .normalize_or_zero();
        if let Some((_, mut transform)) = visuals
            .iter_mut()
            .find(|(visual, _)| visual.0 == hover.seat)
        {
            transform.translation = Val2::px(
                direction.x * hover.amount * 5.0,
                direction.y * hover.amount * 5.0,
            );
            transform.scale = Vec2::splat(1.0 + hover.amount * 0.035);
        }
        if let Some((_, mut border, mut background)) = empty_rings
            .iter_mut()
            .find(|(ring, _, _)| ring.0 == hover.seat)
        {
            border.set_all(if hovered {
                ACCENT.with_alpha(0.88)
            } else {
                MUTED.with_alpha(0.34)
            });
            background.0 = if hovered {
                PANEL_ALT.with_alpha(0.90)
            } else {
                Color::srgba(0.03, 0.12, 0.085, 0.64)
            };
        }
        if let Some((_, mut text, mut color)) = empty_labels
            .iter_mut()
            .find(|(label, _, _)| label.0 == hover.seat)
        {
            let expected_text = if hovered { "点击入座" } else { "空位" };
            let expected_color = if hovered { ACCENT } else { MUTED };
            if text.0 != expected_text {
                text.0 = expected_text.to_owned();
            }
            if color.0 != expected_color {
                color.0 = expected_color;
            }
        }
    }
}

pub(crate) fn handle_lobby_bot_seat_right_click(
    mouse: Res<ButtonInput<MouseButton>>,
    seats: Query<(&Interaction, &LobbySeatHover)>,
    mut client: Option<ResMut<ClientResource>>,
) {
    #[cfg(not(feature = "developer"))]
    let _ = (&mouse, &seats, &mut client);
    #[cfg(feature = "developer")]
    {
        if !mouse.just_pressed(MouseButton::Right) {
            return;
        }
        let Some(seat) = seats
            .iter()
            .find(|(interaction, _)| !matches!(interaction, Interaction::None))
            .map(|(_, hover)| SeatId(hover.seat))
        else {
            return;
        };
        let Some(client) = client.as_deref_mut() else {
            return;
        };
        let occupied = {
            let model = client.0.model();
            let Some(lobby) = model.lobby() else {
                return;
            };
            if model.you() != lobby.host {
                return;
            }
            match lobby
                .players
                .iter()
                .find(|player| player.seat == Some(seat))
            {
                None => true,
                Some(player) if player.profile_id.0 == [0; 32] => false,
                Some(_) => return,
            }
        };
        client
            .0
            .send(ClientCommand::ConfigureBotSeat { seat, occupied });
    }
}

pub(crate) fn update_rule_help_tooltips(
    helps: Query<(&Interaction, &RuleHelp), Changed<Interaction>>,
    mut tooltips: Query<&mut Visibility>,
) {
    for (interaction, help) in &helps {
        if let Ok(mut visibility) = tooltips.get_mut(help.tooltip) {
            *visibility = if matches!(interaction, Interaction::Hovered | Interaction::Pressed) {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

pub(crate) fn table_brightness_fraction(brightness: f32) -> f32 {
    ((normalize_table_brightness(brightness) - MIN_TABLE_BRIGHTNESS)
        / (MAX_TABLE_BRIGHTNESS - MIN_TABLE_BRIGHTNESS))
        .clamp(0.0, 1.0)
}

pub(crate) fn slider_fraction_from_relative_x(relative_x: f32) -> f32 {
    // Bevy 的 RelativeCursorPosition 以节点中心为 0，左右边缘分别为 -0.5 和 0.5。
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
        TableAppearanceSetting::Volume => {
            form.audio_volume = fraction;
        }
    }
}

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
