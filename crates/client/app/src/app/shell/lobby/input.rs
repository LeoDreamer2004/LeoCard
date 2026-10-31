//! 大厅席位与规则提示输入。

use crate::app::presentation::{
    ACCENT, LobbyEmptySeatLabel, LobbyEmptySeatRing, LobbySeatHover, LobbySeatVisual, MUTED,
};
use crate::app::runtime::UiAssets;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;

pub(crate) fn animate_lobby_seat_hover(
    time: Res<Time>,
    assets: Res<UiAssets>,
    mut seats: Query<(&PickingInteraction, &mut LobbySeatHover)>,
    mut visuals: Query<(&LobbySeatVisual, &mut UiTransform)>,
    mut empty_rings: Query<(&LobbyEmptySeatRing, &mut ImageNode)>,
    mut empty_labels: Query<(&LobbyEmptySeatLabel, &mut Text, &mut TextColor)>,
) {
    let response = 1.0 - (-time.delta_secs() * 16.0).exp();
    for (interaction, mut hover) in &mut seats {
        let hovered = !matches!(interaction, PickingInteraction::None);
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
        if let Some((_, mut image)) = empty_rings
            .iter_mut()
            .find(|(ring, _)| ring.0 == hover.seat)
        {
            let expected = if hovered {
                &assets.home.purple_button_compact
            } else {
                &assets.home.button
            };
            if image.image != *expected {
                image.image = expected.clone();
            }
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
