//! 大厅席位与规则提示输入。

use crate::app::presentation::{
    ACCENT, LobbyEmptySeatLabel, LobbyEmptySeatRing, LobbySeatHover, LobbySeatVisual, MUTED,
    PANEL_ALT, RuleHelp,
};
use crate::app::runtime::ClientResource;
use bevy::prelude::*;
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
